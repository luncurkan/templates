package main

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/logger"
)

// URLStore is an in-memory store for URL mappings
type URLStore struct {
	mu   sync.RWMutex
	urls map[string]URLEntry
}

// URLEntry represents a shortened URL entry
type URLEntry struct {
	OriginalURL string    `json:"original_url"`
	ShortCode   string    `json:"short_code"`
	CreatedAt   time.Time `json:"created_at"`
	Visits      int64     `json:"visits"`
}

// ShortenRequest represents the request body for shortening a URL
type ShortenRequest struct {
	URL string `json:"url" form:"url"`
}

// ShortenResponse represents the response for a shortened URL
type ShortenResponse struct {
	ShortURL    string `json:"short_url"`
	ShortCode   string `json:"short_code"`
	OriginalURL string `json:"original_url"`
}

// StatsResponse represents URL statistics
type StatsResponse struct {
	OriginalURL string    `json:"original_url"`
	ShortCode   string    `json:"short_code"`
	CreatedAt   time.Time `json:"created_at"`
	Visits      int64     `json:"visits"`
}

// ErrorResponse represents an error response
type ErrorResponse struct {
	Error string `json:"error"`
}

var store = &URLStore{
	urls: make(map[string]URLEntry),
}

func main() {
	app := fiber.New(fiber.Config{
		AppName: "Go Fiber URL Shortener",
	})

	// Middleware
	app.Use(logger.New())

	// HTML UI
	app.Get("/", indexPage)

	// HTMX endpoints
	app.Post("/api/shorten", shortenURLHtmx)
	app.Get("/api/urls/list", listURLsHtmx)

	// API endpoints
	app.Post("/shorten", shortenURL)
	app.Get("/api/urls", listURLs)
	app.Get("/api/health", healthCheck)

	// Redirect (must be last due to wildcard)
	app.Get("/:code/stats", getStats)
	app.Get("/:code", redirectURL)

	log.Println("Starting server on :3000")
	log.Fatal(app.Listen(":3000"))
}

// indexPage serves the HTML UI
func indexPage(c *fiber.Ctx) error {
	html := `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>URL Shortener</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 2rem;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        h1 {
            color: white;
            text-align: center;
            margin-bottom: 2rem;
            font-size: 2.5rem;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }
        .card {
            background: white;
            border-radius: 16px;
            padding: 2rem;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            margin-bottom: 2rem;
        }
        .form-group {
            display: flex;
            gap: 1rem;
            margin-bottom: 1rem;
        }
        input[type="url"] {
            flex: 1;
            padding: 1rem 1.5rem;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 1rem;
            transition: border-color 0.3s;
        }
        input[type="url"]:focus {
            outline: none;
            border-color: #667eea;
        }
        button {
            padding: 1rem 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 5px 20px rgba(102, 126, 234, 0.4);
        }
        button:active {
            transform: translateY(0);
        }
        .htmx-request button {
            opacity: 0.7;
            cursor: wait;
        }
        #result {
            margin-top: 1rem;
        }
        .result-box {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1.5rem;
            border-left: 4px solid #667eea;
        }
        .result-box a {
            color: #667eea;
            font-weight: 600;
            word-break: break-all;
        }
        .result-box .original {
            color: #666;
            font-size: 0.9rem;
            margin-top: 0.5rem;
            word-break: break-all;
        }
        .error-box {
            background: #fff5f5;
            border-left-color: #e53e3e;
            color: #c53030;
        }
        h2 {
            color: #333;
            margin-bottom: 1rem;
            font-size: 1.25rem;
        }
        .url-list {
            list-style: none;
        }
        .url-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1rem;
            border-bottom: 1px solid #eee;
            gap: 1rem;
        }
        .url-item:last-child {
            border-bottom: none;
        }
        .url-info {
            flex: 1;
            min-width: 0;
        }
        .url-info a {
            color: #667eea;
            font-weight: 600;
            text-decoration: none;
        }
        .url-info a:hover {
            text-decoration: underline;
        }
        .url-info .original {
            color: #666;
            font-size: 0.85rem;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        .url-stats {
            display: flex;
            gap: 1rem;
            font-size: 0.85rem;
            color: #666;
        }
        .stat {
            display: flex;
            align-items: center;
            gap: 0.25rem;
        }
        .empty-state {
            text-align: center;
            color: #666;
            padding: 2rem;
        }
        .copy-btn {
            padding: 0.5rem 1rem;
            font-size: 0.85rem;
            background: #f0f0f0;
            color: #333;
        }
        .copy-btn:hover {
            background: #e0e0e0;
            box-shadow: none;
            transform: none;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔗 URL Shortener</h1>

        <div class="card">
            <form hx-post="/api/shorten" hx-target="#result" hx-swap="innerHTML">
                <div class="form-group">
                    <input type="url" name="url" placeholder="Enter your long URL here..." required>
                    <button type="submit">Shorten</button>
                </div>
            </form>
            <div id="result"></div>
        </div>

        <div class="card">
            <h2>Recent URLs</h2>
            <div id="url-list" hx-get="/api/urls/list" hx-trigger="load, urlCreated from:body" hx-swap="innerHTML">
                <div class="empty-state">Loading...</div>
            </div>
        </div>
    </div>

    <script>
        function copyToClipboard(text, btn) {
            navigator.clipboard.writeText(text).then(() => {
                const original = btn.textContent;
                btn.textContent = 'Copied!';
                setTimeout(() => btn.textContent = original, 2000);
            });
        }
    </script>
</body>
</html>`
	c.Set("Content-Type", "text/html")
	return c.SendString(html)
}

// shortenURLHtmx handles HTMX shorten requests
func shortenURLHtmx(c *fiber.Ctx) error {
	var req ShortenRequest
	if err := c.BodyParser(&req); err != nil {
		return c.SendString(`<div class="result-box error-box">Invalid request</div>`)
	}

	if req.URL == "" {
		return c.SendString(`<div class="result-box error-box">Please enter a URL</div>`)
	}

	shortCode, err := generateShortCode(6)
	if err != nil {
		return c.SendString(`<div class="result-box error-box">Failed to generate short code</div>`)
	}

	entry := URLEntry{
		OriginalURL: req.URL,
		ShortCode:   shortCode,
		CreatedAt:   time.Now(),
		Visits:      0,
	}

	store.mu.Lock()
	store.urls[shortCode] = entry
	store.mu.Unlock()

	baseURL := c.BaseURL()
	shortURL := baseURL + "/" + shortCode

	html := fmt.Sprintf(`
		<div class="result-box" hx-trigger="load" hx-swap-oob="true" hx-get="/api/urls/list" hx-target="#url-list">
			<a href="%s" target="_blank">%s</a>
			<button class="copy-btn" onclick="copyToClipboard('%s', this)" style="margin-left: 1rem;">Copy</button>
			<div class="original">Original: %s</div>
		</div>
		<script>document.body.dispatchEvent(new Event('urlCreated'))</script>
	`, shortURL, shortURL, shortURL, req.URL)

	return c.SendString(html)
}

// listURLsHtmx returns HTML list of URLs for HTMX
func listURLsHtmx(c *fiber.Ctx) error {
	store.mu.RLock()
	defer store.mu.RUnlock()

	if len(store.urls) == 0 {
		return c.SendString(`<div class="empty-state">No URLs shortened yet. Try creating one above!</div>`)
	}

	html := `<ul class="url-list">`
	baseURL := c.BaseURL()

	for _, entry := range store.urls {
		shortURL := baseURL + "/" + entry.ShortCode
		html += fmt.Sprintf(`
			<li class="url-item">
				<div class="url-info">
					<a href="%s" target="_blank">%s</a>
					<div class="original">%s</div>
				</div>
				<div class="url-stats">
					<span class="stat">👁 %d visits</span>
				</div>
				<button class="copy-btn" onclick="copyToClipboard('%s', this)">Copy</button>
			</li>
		`, shortURL, shortURL, entry.OriginalURL, entry.Visits, shortURL)
	}

	html += `</ul>`
	return c.SendString(html)
}

// healthCheck returns server health status
func healthCheck(c *fiber.Ctx) error {
	return c.JSON(fiber.Map{
		"status":  "ok",
		"message": "Go Fiber URL Shortener is running",
	})
}

// shortenURL creates a shortened URL (JSON API)
func shortenURL(c *fiber.Ctx) error {
	var req ShortenRequest
	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(ErrorResponse{
			Error: "Invalid request body",
		})
	}

	if req.URL == "" {
		return c.Status(fiber.StatusBadRequest).JSON(ErrorResponse{
			Error: "URL is required",
		})
	}

	shortCode, err := generateShortCode(6)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(ErrorResponse{
			Error: "Failed to generate short code",
		})
	}

	entry := URLEntry{
		OriginalURL: req.URL,
		ShortCode:   shortCode,
		CreatedAt:   time.Now(),
		Visits:      0,
	}

	store.mu.Lock()
	store.urls[shortCode] = entry
	store.mu.Unlock()

	baseURL := c.BaseURL()
	shortURL := baseURL + "/" + shortCode

	return c.Status(fiber.StatusCreated).JSON(ShortenResponse{
		ShortURL:    shortURL,
		ShortCode:   shortCode,
		OriginalURL: req.URL,
	})
}

// redirectURL redirects to the original URL
func redirectURL(c *fiber.Ctx) error {
	code := c.Params("code")

	store.mu.Lock()
	entry, exists := store.urls[code]
	if !exists {
		store.mu.Unlock()
		return c.Status(fiber.StatusNotFound).JSON(ErrorResponse{
			Error: "Short URL not found",
		})
	}

	entry.Visits++
	store.urls[code] = entry
	store.mu.Unlock()

	return c.Redirect(entry.OriginalURL, fiber.StatusMovedPermanently)
}

// getStats returns statistics for a shortened URL
func getStats(c *fiber.Ctx) error {
	code := c.Params("code")

	store.mu.RLock()
	entry, exists := store.urls[code]
	store.mu.RUnlock()

	if !exists {
		return c.Status(fiber.StatusNotFound).JSON(ErrorResponse{
			Error: "Short URL not found",
		})
	}

	return c.JSON(StatsResponse{
		OriginalURL: entry.OriginalURL,
		ShortCode:   entry.ShortCode,
		CreatedAt:   entry.CreatedAt,
		Visits:      entry.Visits,
	})
}

// listURLs returns all shortened URLs (JSON API)
func listURLs(c *fiber.Ctx) error {
	store.mu.RLock()
	defer store.mu.RUnlock()

	urls := make([]StatsResponse, 0, len(store.urls))
	for _, entry := range store.urls {
		urls = append(urls, StatsResponse{
			OriginalURL: entry.OriginalURL,
			ShortCode:   entry.ShortCode,
			CreatedAt:   entry.CreatedAt,
			Visits:      entry.Visits,
		})
	}

	return c.JSON(fiber.Map{
		"urls":  urls,
		"count": len(urls),
	})
}

// generateShortCode generates a random short code
func generateShortCode(length int) (string, error) {
	bytes := make([]byte, length)
	if _, err := rand.Read(bytes); err != nil {
		return "", err
	}
	return hex.EncodeToString(bytes)[:length], nil
}

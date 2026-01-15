package main

import (
	"database/sql"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/golang-jwt/jwt/v5"
	_ "github.com/mattn/go-sqlite3"
	"golang.org/x/crypto/bcrypt"
)

var (
	db                        *sql.DB
	jwtSecret                 []byte
	accessTokenExpireMinutes  int
	refreshTokenExpireDays    int
)

type User struct {
	ID        int64     `json:"id"`
	Email     string    `json:"email"`
	Password  string    `json:"-"`
	CreatedAt time.Time `json:"created_at"`
}

type RegisterRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

type RefreshRequest struct {
	RefreshToken string `json:"refresh_token"`
}

type TokenResponse struct {
	AccessToken  string      `json:"access_token"`
	RefreshToken string      `json:"refresh_token"`
	User         UserResponse `json:"user"`
}

type UserResponse struct {
	ID    int64  `json:"id"`
	Email string `json:"email"`
}

func getEnv(key, defaultVal string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return defaultVal
}

func getEnvInt(key string, defaultVal int) int {
	if val := os.Getenv(key); val != "" {
		if i, err := strconv.Atoi(val); err == nil {
			return i
		}
	}
	return defaultVal
}

func createAccessToken(userID int64, email string) (string, error) {
	claims := jwt.MapClaims{
		"user_id": userID,
		"email":   email,
		"type":    "access",
		"exp":     time.Now().Add(time.Duration(accessTokenExpireMinutes) * time.Minute).Unix(),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(jwtSecret)
}

func createRefreshToken(userID int64) (string, error) {
	claims := jwt.MapClaims{
		"user_id": userID,
		"type":    "refresh",
		"exp":     time.Now().Add(time.Duration(refreshTokenExpireDays) * 24 * time.Hour).Unix(),
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(jwtSecret)
}

func verifyToken(tokenString string, expectedType string) (jwt.MapClaims, error) {
	token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
		return jwtSecret, nil
	})
	if err != nil {
		return nil, err
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok || !token.Valid {
		return nil, fmt.Errorf("invalid token")
	}

	if claims["type"] != expectedType {
		return nil, fmt.Errorf("invalid token type")
	}

	return claims, nil
}

func authMiddleware(c *fiber.Ctx) error {
	authHeader := c.Get("Authorization")
	if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
		return c.Status(401).JSON(fiber.Map{"error": "Missing authorization header"})
	}

	tokenString := strings.TrimPrefix(authHeader, "Bearer ")
	claims, err := verifyToken(tokenString, "access")
	if err != nil {
		return c.Status(401).JSON(fiber.Map{"error": "Invalid or expired token"})
	}

	c.Locals("user_id", int64(claims["user_id"].(float64)))
	c.Locals("email", claims["email"].(string))
	return c.Next()
}

func main() {
	// Environment
	host := getEnv("HOST", "0.0.0.0")
	port := getEnv("PORT", "3000")
	dbPath := getEnv("DATABASE_PATH", "./data.db")
	jwtSecret = []byte(getEnv("JWT_SECRET", "change-me"))
	accessTokenExpireMinutes = getEnvInt("ACCESS_TOKEN_EXPIRE_MINUTES", 15)
	refreshTokenExpireDays = getEnvInt("REFRESH_TOKEN_EXPIRE_DAYS", 7)

	// Database
	var err error
	db, err = sql.Open("sqlite3", dbPath)
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	_, err = db.Exec(`
		CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			email TEXT UNIQUE NOT NULL,
			password TEXT NOT NULL,
			created_at DATETIME DEFAULT CURRENT_TIMESTAMP
		)
	`)
	if err != nil {
		log.Fatal(err)
	}

	// App
	app := fiber.New()
	app.Use(logger.New())
	app.Use(cors.New())

	// Health check
	app.Get("/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"status": "ok"})
	})

	// Register
	app.Post("/api/auth/register", func(c *fiber.Ctx) error {
		var req RegisterRequest
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request"})
		}

		if len(req.Password) < 8 {
			return c.Status(400).JSON(fiber.Map{"error": "Password must be at least 8 characters"})
		}

		var existingID int64
		err := db.QueryRow("SELECT id FROM users WHERE email = ?", req.Email).Scan(&existingID)
		if err == nil {
			return c.Status(409).JSON(fiber.Map{"error": "Email already registered"})
		}

		hashedPassword, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Internal error"})
		}

		result, err := db.Exec("INSERT INTO users (email, password) VALUES (?, ?)", req.Email, string(hashedPassword))
		if err != nil {
			return c.Status(500).JSON(fiber.Map{"error": "Internal error"})
		}

		userID, _ := result.LastInsertId()

		accessToken, _ := createAccessToken(userID, req.Email)
		refreshToken, _ := createRefreshToken(userID)

		return c.Status(201).JSON(TokenResponse{
			AccessToken:  accessToken,
			RefreshToken: refreshToken,
			User:         UserResponse{ID: userID, Email: req.Email},
		})
	})

	// Login
	app.Post("/api/auth/login", func(c *fiber.Ctx) error {
		var req LoginRequest
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request"})
		}

		var user User
		err := db.QueryRow("SELECT id, email, password FROM users WHERE email = ?", req.Email).
			Scan(&user.ID, &user.Email, &user.Password)
		if err != nil {
			return c.Status(401).JSON(fiber.Map{"error": "Invalid credentials"})
		}

		if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(req.Password)); err != nil {
			return c.Status(401).JSON(fiber.Map{"error": "Invalid credentials"})
		}

		accessToken, _ := createAccessToken(user.ID, user.Email)
		refreshToken, _ := createRefreshToken(user.ID)

		return c.JSON(TokenResponse{
			AccessToken:  accessToken,
			RefreshToken: refreshToken,
			User:         UserResponse{ID: user.ID, Email: user.Email},
		})
	})

	// Refresh
	app.Post("/api/auth/refresh", func(c *fiber.Ctx) error {
		var req RefreshRequest
		if err := c.BodyParser(&req); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": "Invalid request"})
		}

		claims, err := verifyToken(req.RefreshToken, "refresh")
		if err != nil {
			return c.Status(401).JSON(fiber.Map{"error": "Invalid or expired token"})
		}

		userID := int64(claims["user_id"].(float64))

		var user User
		err = db.QueryRow("SELECT id, email FROM users WHERE id = ?", userID).Scan(&user.ID, &user.Email)
		if err != nil {
			return c.Status(401).JSON(fiber.Map{"error": "User not found"})
		}

		accessToken, _ := createAccessToken(user.ID, user.Email)
		refreshToken, _ := createRefreshToken(user.ID)

		return c.JSON(fiber.Map{
			"access_token":  accessToken,
			"refresh_token": refreshToken,
		})
	})

	// Get current user (protected)
	app.Get("/api/auth/me", authMiddleware, func(c *fiber.Ctx) error {
		userID := c.Locals("user_id").(int64)

		var user User
		err := db.QueryRow("SELECT id, email FROM users WHERE id = ?", userID).Scan(&user.ID, &user.Email)
		if err != nil {
			return c.Status(401).JSON(fiber.Map{"error": "User not found"})
		}

		return c.JSON(fiber.Map{
			"user": UserResponse{ID: user.ID, Email: user.Email},
		})
	})

	log.Printf("Server starting on http://%s:%s", host, port)
	log.Fatal(app.Listen(fmt.Sprintf("%s:%s", host, port)))
}

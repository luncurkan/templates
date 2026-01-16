package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"sync"
	"time"

	"github.com/gofiber/contrib/websocket"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
)

type Client struct {
	Conn     *websocket.Conn
	Username string
	Room     string
}

type Message struct {
	Type      string `json:"type"`
	Username  string `json:"username,omitempty"`
	Content   string `json:"content,omitempty"`
	Timestamp string `json:"timestamp,omitempty"`
}

var (
	rooms = make(map[string]map[*Client]bool)
	mu    sync.RWMutex
)

func getEnv(key, defaultVal string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return defaultVal
}

func getRoom(name string) map[*Client]bool {
	mu.Lock()
	defer mu.Unlock()
	if rooms[name] == nil {
		rooms[name] = make(map[*Client]bool)
	}
	return rooms[name]
}

func addClient(client *Client) {
	mu.Lock()
	defer mu.Unlock()
	if rooms[client.Room] == nil {
		rooms[client.Room] = make(map[*Client]bool)
	}
	rooms[client.Room][client] = true
}

func removeClient(client *Client) {
	mu.Lock()
	defer mu.Unlock()
	if rooms[client.Room] != nil {
		delete(rooms[client.Room], client)
	}
}

func broadcast(room string, msg Message, exclude *Client) {
	mu.RLock()
	clients := rooms[room]
	mu.RUnlock()

	data, _ := json.Marshal(msg)
	for client := range clients {
		if client != exclude {
			client.Conn.WriteMessage(websocket.TextMessage, data)
		}
	}
}

func handleWebSocket(c *websocket.Conn, room string) {
	client := &Client{
		Conn:     c,
		Username: "Anonymous",
		Room:     room,
	}

	addClient(client)
	defer func() {
		removeClient(client)
		broadcast(room, Message{
			Type:      "message",
			Username:  "System",
			Content:   fmt.Sprintf("%s left the room", client.Username),
			Timestamp: time.Now().UTC().Format(time.RFC3339),
		}, nil)
		c.Close()
	}()

	broadcast(room, Message{
		Type:      "message",
		Username:  "System",
		Content:   "A user joined the room",
		Timestamp: time.Now().UTC().Format(time.RFC3339),
	}, client)

	for {
		_, msgBytes, err := c.ReadMessage()
		if err != nil {
			break
		}

		var msg Message
		if err := json.Unmarshal(msgBytes, &msg); err != nil {
			continue
		}

		switch msg.Type {
		case "username":
			if msg.Username != "" {
				client.Username = msg.Username
			}
		case "message":
			if msg.Content != "" {
				broadcast(room, Message{
					Type:      "message",
					Username:  client.Username,
					Content:   msg.Content,
					Timestamp: time.Now().UTC().Format(time.RFC3339),
				}, nil)
			}
		}
	}
}

const chatHTML = `<!DOCTYPE html>
<html>
<head>
  <title>WebSocket Chat</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: system-ui; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; display: flex; justify-content: center; align-items: center; }
    .chat { background: white; border-radius: 12px; width: 400px; max-width: 90vw; box-shadow: 0 20px 60px rgba(0,0,0,0.3); overflow: hidden; }
    .header { background: #667eea; color: white; padding: 20px; text-align: center; }
    .messages { height: 300px; overflow-y: auto; padding: 20px; }
    .message { margin: 10px 0; padding: 10px; background: #f5f5f5; border-radius: 8px; }
    .message .user { font-weight: bold; color: #667eea; }
    .message .time { font-size: 0.75em; color: #999; }
    .input-area { display: flex; padding: 20px; gap: 10px; border-top: 1px solid #eee; }
    input { flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 6px; }
    button { padding: 10px 20px; background: #667eea; color: white; border: none; border-radius: 6px; cursor: pointer; }
    button:hover { background: #5a6fd6; }
    .status { padding: 10px; text-align: center; font-size: 0.9em; color: #666; }
  </style>
</head>
<body>
  <div class="chat">
    <div class="header"><h2>WebSocket Chat</h2></div>
    <div class="status" id="status">Connecting...</div>
    <div class="messages" id="messages"></div>
    <div class="input-area">
      <input type="text" id="message" placeholder="Type a message..." onkeypress="if(event.key==='Enter')sendMessage()">
      <button onclick="sendMessage()">Send</button>
    </div>
  </div>
  <script>
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(protocol + '//' + location.host + '/ws');
    const messages = document.getElementById('messages');
    const status = document.getElementById('status');
    let username = 'User' + Math.floor(Math.random() * 1000);

    ws.onopen = () => {
      status.textContent = 'Connected as ' + username;
      status.style.color = '#4caf50';
      ws.send(JSON.stringify({ type: 'username', username }));
    };

    ws.onclose = () => {
      status.textContent = 'Disconnected';
      status.style.color = '#f44336';
    };

    ws.onmessage = (e) => {
      const data = JSON.parse(e.data);
      if (data.type === 'message') {
        const div = document.createElement('div');
        div.className = 'message';
        div.innerHTML = '<span class="user">' + data.username + '</span> <span class="time">' + new Date(data.timestamp).toLocaleTimeString() + '</span><div>' + data.content + '</div>';
        messages.appendChild(div);
        messages.scrollTop = messages.scrollHeight;
      }
    };

    function sendMessage() {
      const input = document.getElementById('message');
      if (input.value.trim()) {
        ws.send(JSON.stringify({ type: 'message', content: input.value }));
        input.value = '';
      }
    }
  </script>
</body>
</html>`

func main() {
	host := getEnv("HOST", "0.0.0.0")
	port := getEnv("PORT", "3000")

	app := fiber.New()
	app.Use(logger.New())
	app.Use(cors.New())

	// Health check
	app.Get("/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"status": "ok"})
	})

	// Chat UI
	app.Get("/", func(c *fiber.Ctx) error {
		c.Set("Content-Type", "text/html")
		return c.SendString(chatHTML)
	})

	// WebSocket upgrade middleware
	app.Use("/ws", func(c *fiber.Ctx) error {
		if websocket.IsWebSocketUpgrade(c) {
			return c.Next()
		}
		return fiber.ErrUpgradeRequired
	})

	// WebSocket handlers
	app.Get("/ws", websocket.New(func(c *websocket.Conn) {
		handleWebSocket(c, "general")
	}))

	app.Get("/ws/:room", websocket.New(func(c *websocket.Conn) {
		room := c.Params("room")
		if room == "" {
			room = "general"
		}
		handleWebSocket(c, room)
	}))

	log.Printf("Server starting on http://%s:%s", host, port)
	log.Fatal(app.Listen(fmt.Sprintf("%s:%s", host, port)))
}

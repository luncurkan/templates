import { Hono } from "hono";
import { serve } from "@hono/node-server";
import { cors } from "hono/cors";
import { logger } from "hono/logger";
import { createNodeWebSocket } from "@hono/node-ws";
import type { WSContext } from "hono/ws";

// Environment
const HOST = process.env.HOST || "0.0.0.0";
const PORT = parseInt(process.env.PORT || "3000");

// Types
interface Client {
  ws: WSContext;
  username: string;
  room: string;
}

interface Message {
  type: string;
  username?: string;
  content?: string;
  timestamp?: string;
}

// State
const rooms = new Map<string, Set<Client>>();

function getRoom(roomName: string): Set<Client> {
  if (!rooms.has(roomName)) {
    rooms.set(roomName, new Set());
  }
  return rooms.get(roomName)!;
}

function broadcast(room: string, message: Message, exclude?: WSContext) {
  const clients = getRoom(room);
  const data = JSON.stringify(message);
  for (const client of clients) {
    if (client.ws !== exclude && client.ws.readyState === 1) {
      client.ws.send(data);
    }
  }
}

// App
const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

app.use("*", logger());
app.use("*", cors());

// Health check
app.get("/health", (c) => c.json({ status: "ok" }));

// Chat UI
app.get("/", (c) => {
  return c.html(`
    <!DOCTYPE html>
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
        const wsProtocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
        const ws = new WebSocket(wsProtocol + '//' + location.host + '/ws');
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
    </html>
  `);
});

// WebSocket handler
const wsHandler = upgradeWebSocket((c) => {
  const roomName = c.req.param("room") || "general";
  let client: Client | null = null;

  return {
    onOpen: (_event, ws) => {
      client = { ws, username: "Anonymous", room: roomName };
      getRoom(roomName).add(client);
      broadcast(roomName, {
        type: "message",
        username: "System",
        content: `A user joined the room`,
        timestamp: new Date().toISOString(),
      });
    },

    onMessage: (event, ws) => {
      try {
        const data = JSON.parse(event.data.toString()) as Message;

        if (data.type === "username" && data.username && client) {
          client.username = data.username;
        } else if (data.type === "message" && data.content && client) {
          broadcast(roomName, {
            type: "message",
            username: client.username,
            content: data.content,
            timestamp: new Date().toISOString(),
          });
        }
      } catch {
        // Ignore invalid messages
      }
    },

    onClose: () => {
      if (client) {
        getRoom(roomName).delete(client);
        broadcast(roomName, {
          type: "message",
          username: "System",
          content: `${client.username} left the room`,
          timestamp: new Date().toISOString(),
        });
      }
    },
  };
});

app.get("/ws", wsHandler);
app.get("/ws/:room", wsHandler);

// Start server
const server = serve({ fetch: app.fetch, hostname: HOST, port: PORT });
injectWebSocket(server);

console.log(`Server running on http://${HOST}:${PORT}`);

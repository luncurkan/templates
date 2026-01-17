import { Elysia, t } from "elysia";

// Environment
const HOST = process.env.HOST || "0.0.0.0";
const PORT = parseInt(process.env.PORT || "3000");

// Types
interface Client {
  ws: any;
  username: string;
  room: string;
}

interface Message {
  type: string;
  username?: string;
  content?: string;
  timestamp?: string;
}

// Logging utility
const log = {
  info: (context: string, message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`[${timestamp}] [INFO] [${context}] ${message}`, data ? JSON.stringify(data) : '');
  },
  error: (context: string, message: string, error?: any) => {
    const timestamp = new Date().toISOString();
    console.error(`[${timestamp}] [ERROR] [${context}] ${message}`, error || '');
  },
  debug: (context: string, message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`[${timestamp}] [DEBUG] [${context}] ${message}`, data ? JSON.stringify(data) : '');
  },
};

// State
const rooms = new Map<string, Set<Client>>();
const clients = new Map<any, Client>(); // Map WebSocket to Client

function getRoom(roomName: string): Set<Client> {
  if (!rooms.has(roomName)) {
    rooms.set(roomName, new Set());
    log.info('Room', `Created new room: ${roomName}`);
  }
  return rooms.get(roomName)!;
}

function broadcast(room: string, message: Message, exclude?: any) {
  const clients = getRoom(room);
  const data = JSON.stringify(message);
  let sentCount = 0;
  for (const client of clients) {
    if (client.ws !== exclude && client.ws.readyState === 1) {
      client.ws.send(data);
      sentCount++;
    }
  }
  log.info('Broadcast', `Sent to ${sentCount}/${clients.size} clients in room "${room}"`, { type: message.type, username: message.username });
}

// HTML UI
const chatHtml = `
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
      <input type="text" id="message" placeholder="Type a message..." onkeypress="if(event.key==='Enter' && isConnected) sendMessage()">
      <button onclick="sendMessage()">Send</button>
    </div>
  </div>
  <script>
    // Detect protocol and construct WebSocket URL with proper error handling
    const getWebSocketUrl = () => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;
        return \`\${protocol}//\${host}/ws\`;
    };

    const ws = new WebSocket(getWebSocketUrl());
    const messages = document.getElementById('messages');
    const status = document.getElementById('status');
    let username = 'User' + Math.floor(Math.random() * 10000);

    // Add connection state tracking
    let isConnected = false;

    // Handle WebSocket open
    ws.onopen = () => {
        isConnected = true;
        status.textContent = 'Connected as ' + username;
        status.style.color = '#4caf50';
        console.log('WebSocket connected to:', getWebSocketUrl());

        // Send username immediately
        ws.send(JSON.stringify({
            type: 'username',
            username: username
        }));
    };

    // Handle WebSocket close
    ws.onclose = () => {
        isConnected = false;
        status.textContent = 'Disconnected - Attempting to reconnect...';
        status.style.color = '#f44336';
        console.log('WebSocket disconnected');

        // Attempt to reconnect after 3 seconds
        setTimeout(() => {
            location.reload();
        }, 3000);
    };

    // Handle WebSocket errors
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        status.textContent = 'Connection error - Please refresh the page';
        status.style.color = '#ff9800';
    };

    // Handle incoming messages
    ws.onmessage = (e) => {
        try {
            const data = JSON.parse(e.data);
            if (data.type === 'message') {
                const div = document.createElement('div');
                div.className = 'message';
                const timeStr = new Date(data.timestamp).toLocaleTimeString();
                div.innerHTML = \`<span class="user">\${data.username}</span> <span class="time">\${timeStr}</span><div>\${data.content}</div>\`;
                messages.appendChild(div);
                messages.scrollTop = messages.scrollHeight;
            }
        } catch (err) {
            console.error('Error parsing message:', err);
        }
    };

    // Send message function with connection check
    function sendMessage() {
        const input = document.getElementById('message');
        if (!input.value.trim()) return;

        if (!isConnected || ws.readyState !== WebSocket.OPEN) {
            alert('WebSocket not connected. Please refresh the page.');
            return;
        }

        try {
            ws.send(JSON.stringify({
                type: 'message',
                content: input.value
            }));
            input.value = '';
        } catch (err) {
            console.error('Error sending message:', err);
        }
    }
  </script>
</body>
</html>
`;

// App
const app = new Elysia()
  // Health check
  .get("/health", () => ({ status: "ok" }))

  // Chat UI
  .get("/", () => new Response(chatHtml, { headers: { "Content-Type": "text/html" } }))

  // WebSocket handlers
  .ws("/ws", {
    body: t.Object({
      type: t.String(),
      username: t.Optional(t.String()),
      content: t.Optional(t.String()),
    }),
    open(ws) {
      const client: Client = { ws: ws.raw, username: "Anonymous", room: "general" };
      clients.set(ws.raw, client);
      getRoom("general").add(client);
      log.info('WS /ws', `Client connected`, { room: "general", totalClients: getRoom("general").size });
      broadcast("general", {
        type: "message",
        username: "System",
        content: "A user joined the room",
        timestamp: new Date().toISOString(),
      });
    },
    message(ws, data) {
      const client = clients.get(ws.raw);
      log.debug('WS /ws', `Message received`, { type: data.type, hasClient: !!client, rawData: data });

      if (!client) {
        log.error('WS /ws', 'No client found in clients map');
        return;
      }

      if (data.type === "username" && data.username) {
        const oldUsername = client.username;
        client.username = data.username;
        log.info('WS /ws', `Username set`, { from: oldUsername, to: data.username, room: client.room });
      } else if (data.type === "message" && data.content) {
        log.info('WS /ws', `Chat message`, { username: client.username, content: data.content, room: client.room });
        broadcast(client.room, {
          type: "message",
          username: client.username,
          content: data.content,
          timestamp: new Date().toISOString(),
        });
      } else {
        log.debug('WS /ws', `Unhandled message type`, { type: data.type, data });
      }
    },
    close(ws) {
      const client = clients.get(ws.raw);
      if (client) {
        getRoom(client.room).delete(client);
        clients.delete(ws.raw);
        log.info('WS /ws', `Client disconnected`, { username: client.username, room: client.room, remainingClients: getRoom(client.room).size });
        broadcast(client.room, {
          type: "message",
          username: "System",
          content: `${client.username} left the room`,
          timestamp: new Date().toISOString(),
        });
      }
    },
  })

  .ws("/ws/:room", {
    body: t.Object({
      type: t.String(),
      username: t.Optional(t.String()),
      content: t.Optional(t.String()),
    }),
    open(ws) {
      const room = (ws.data as any)?.params?.room || "general";
      const client: Client = { ws: ws.raw, username: "Anonymous", room };
      clients.set(ws.raw, client);
      getRoom(room).add(client);
      log.info('WS /ws/:room', `Client connected`, { room, totalClients: getRoom(room).size });
      broadcast(room, {
        type: "message",
        username: "System",
        content: "A user joined the room",
        timestamp: new Date().toISOString(),
      });
    },
    message(ws, data) {
      const client = clients.get(ws.raw);
      log.debug('WS /ws/:room', `Message received`, { type: data.type, hasClient: !!client, rawData: data });

      if (!client) {
        log.error('WS /ws/:room', 'No client found in clients map');
        return;
      }

      if (data.type === "username" && data.username) {
        const oldUsername = client.username;
        client.username = data.username;
        log.info('WS /ws/:room', `Username set`, { from: oldUsername, to: data.username, room: client.room });
      } else if (data.type === "message" && data.content) {
        log.info('WS /ws/:room', `Chat message`, { username: client.username, content: data.content, room: client.room });
        broadcast(client.room, {
          type: "message",
          username: client.username,
          content: data.content,
          timestamp: new Date().toISOString(),
        });
      } else {
        log.debug('WS /ws/:room', `Unhandled message type`, { type: data.type, data });
      }
    },
    close(ws) {
      const client = clients.get(ws.raw);
      if (client) {
        getRoom(client.room).delete(client);
        clients.delete(ws.raw);
        log.info('WS /ws/:room', `Client disconnected`, { username: client.username, room: client.room, remainingClients: getRoom(client.room).size });
        broadcast(client.room, {
          type: "message",
          username: "System",
          content: `${client.username} left the room`,
          timestamp: new Date().toISOString(),
        });
      }
    },
  })

  .listen({ hostname: HOST, port: PORT });

log.info('Server', `Started on http://${HOST}:${PORT}`, { host: HOST, port: PORT });

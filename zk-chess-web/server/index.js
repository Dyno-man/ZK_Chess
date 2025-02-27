const express = require('express');
const http = require('http');
const { Server } = require('socket.io');
const next = require('next');
const { initSocket } = require('./socket');

const dev = process.env.NODE_ENV !== 'production';
const app = next({ dev });
const handle = app.getRequestHandler();

app.prepare().then(() => {
  const server = express();
  const httpServer = http.createServer(server);
  
  // Create a single instance of Socket.IO server
  const io = new Server(httpServer);
  
  // Initialize socket handlers
  initSocket(io);
  
  // Handle Next.js requests
  server.all('*', (req, res) => {
    return handle(req, res);
  });
  
  const PORT = process.env.PORT || 3000;
  httpServer.listen(PORT, (err) => {
    if (err) throw err;
    console.log(`> Ready on http://localhost:${PORT}`);
  });
}); 
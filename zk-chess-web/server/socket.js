// Import the game storage module
const { loadGames, saveGames } = require('./gameStorage');

// Load games from storage
const games = loadGames();

// Track active socket connections
const activeSockets = new Map();

// Debug function to log the current state of all games
function logGamesState() {
  console.log('===== CURRENT GAMES STATE =====');
  console.log(`Total games stored: ${games.size}`);
  
  games.forEach((game, gameId) => {
    console.log(`Game ${gameId}:`);
    console.log(`  White player: ${game.white || 'Not joined'}`);
    console.log(`  Black player: ${game.black || 'Not joined'}`);
    console.log(`  Created at: ${game.createdAt}`);
    console.log(`  Available: ${!game.black ? 'Yes' : 'No'}`);
    if (game.creatorDisconnected) {
      console.log(`  Creator disconnected: Yes`);
    }
  });
  
  console.log('===============================');
}

function generateGameId() {
  return Math.random().toString(36).substring(2, 8);
}

// Helper function to get available games
function getAvailableGames() {
  const availableGames = [];
  games.forEach((game, gameId) => {
    // A game is available if it has no black player OR if the creator disconnected
    if (!game.black || game.creatorDisconnected) {
      availableGames.push({
        gameId,
        createdAt: game.createdAt
      });
    }
  });
  
  console.log(`Available games: ${availableGames.length}`);
  availableGames.forEach(game => {
    console.log(`  Game ${game.gameId}, created at ${new Date(game.createdAt).toLocaleTimeString()}`);
  });
  
  return availableGames;
}

// Add a test game for debugging
function addTestGame(io, socket) {
  try {
    const gameId = 'test-' + Math.random().toString(36).substring(2, 8);
    const now = new Date();
    
    games.set(gameId, {
      white: 'test-player-' + now.getTime(),
      black: null,
      createdAt: now
    });
    
    console.log(`Added test game: ${gameId}`);
    
    // Save games to storage
    saveGames(games);
    
    // Get updated games list
    const updatedGames = getAvailableGames();
    
    // Send confirmation to the requesting client first
    if (socket) {
      console.log(`Sending testGameAdded confirmation to client ${socket.id}`);
      socket.emit('testGameAdded', { gameId });
    }
    
    // Then broadcast the updated games list to all clients
    if (io) {
      console.log(`Broadcasting updated game list after adding test game: ${updatedGames.length}`);
      io.emit('availableGames', updatedGames);
    }
    
    return gameId;
  } catch (error) {
    console.error('Error adding test game:', error);
    if (socket) {
      socket.emit('error', 'Failed to add test game');
    }
    return null;
  }
}

// Clear all games (for testing/admin purposes)
function clearAllGames(io, socket) {
  try {
    const gameCount = games.size;
    console.log(`Clearing all ${gameCount} games`);
    
    // Clear the games Map
    games.clear();
    
    // Save empty games to storage
    saveGames(games);
    
    // Log the updated state
    logGamesState();
    
    // Send confirmation to the requesting client
    if (socket) {
      console.log(`Sending gamesCleared confirmation to client ${socket.id}`);
      socket.emit('gamesCleared', { success: true, count: gameCount });
    }
    
    // Broadcast empty games list to all clients
    if (io) {
      console.log('Broadcasting empty game list after clearing all games');
      io.emit('availableGames', []);
    }
    
    return true;
  } catch (error) {
    console.error('Error clearing games:', error);
    if (socket) {
      socket.emit('gamesCleared', { success: false, error: 'Failed to clear games' });
    }
    return false;
  }
}

function initSocket(io) {
  // Log initial state
  console.log('Socket.io server initialized');
  logGamesState();
  
  // Add a test game on server start if there are no games
  if (games.size === 0) {
    console.log('No games found, adding a test game');
    addTestGame(io);
  }

  io.on('connection', (socket) => {
    console.log('New client connected:', socket.id);
    
    // Track this socket connection
    activeSockets.set(socket.id, { connected: true, timestamp: new Date() });
    
    // Send available games when client connects
    const availableGames = getAvailableGames();
    console.log(`Sending available games to new client: ${availableGames.length}`);
    socket.emit('availableGames', availableGames);

    // Handle game creation
    socket.on('createGame', () => {
      try {
        const gameId = generateGameId();
        console.log(`Game created: ${gameId} by player ${socket.id}`);
        
        // Store the game with the current timestamp
        const now = new Date();
        games.set(gameId, {
          white: socket.id,
          black: null,
          createdAt: now,
          // Add a flag to track if the creator is still connected
          creatorConnected: true
        });
        
        // Save games to storage
        saveGames(games);
        
        // Join the socket to a room with the game ID
        socket.join(gameId);
        
        // Notify the creator
        socket.emit('gameCreated', { gameId });
        socket.emit('playerAssigned', { color: 'white' });
        
        // Log the updated state
        logGamesState();
        
        // Get updated games list
        const updatedGames = getAvailableGames();
        
        // Broadcast updated game list to all clients
        console.log(`Broadcasting updated game list after creation: ${updatedGames.length}`);
        io.emit('availableGames', updatedGames);
      } catch (error) {
        console.error('Error creating game:', error);
        socket.emit('error', 'Failed to create game');
      }
    });
    
    // Handle game joining
    socket.on('joinGame', (gameId) => {
      try {
        console.log(`Player ${socket.id} attempting to join game ${gameId}`);
        
        const game = games.get(gameId);
        if (!game) {
          console.log(`Game ${gameId} not found`);
          socket.emit('error', 'Game not found');
          return;
        } 
        
        // If the creator disconnected, allow the new player to take over as white
        if (game.creatorDisconnected) {
          console.log(`Creator disconnected, assigning new player ${socket.id} as white`);
          game.white = socket.id;
          game.creatorDisconnected = false;
          game.creatorConnected = true;
          
          // Join the socket to the game room
          socket.join(gameId);
          
          // Notify the joining player they're white
          socket.emit('playerAssigned', { color: 'white' });
          
          // Save games to storage
          saveGames(games);
          
          // Log the updated state
          logGamesState();
          
          // Get updated games list
          const updatedGames = getAvailableGames();
          
          // Broadcast updated game list to all clients
          console.log(`Broadcasting updated game list after white player reassignment: ${updatedGames.length}`);
          io.emit('availableGames', updatedGames);
          return;
        }
        
        // Normal case - check if the game is full
        if (game.black) {
          console.log(`Game ${gameId} is already full`);
          socket.emit('error', 'Game is full');
          return;
        }
        
        // Check if white player is still connected
        const whiteSocketId = game.white;
        const whiteSocketInfo = activeSockets.get(whiteSocketId);
        
        if (!whiteSocketInfo || !whiteSocketInfo.connected) {
          console.log(`White player ${whiteSocketId} is not connected, marking game as creator disconnected`);
          game.creatorDisconnected = true;
          game.creatorConnected = false;
          
          // Let this player take over as white
          game.white = socket.id;
          game.creatorDisconnected = false;
          game.creatorConnected = true;
          
          // Join the socket to the game room
          socket.join(gameId);
          
          // Notify the joining player they're white
          socket.emit('playerAssigned', { color: 'white' });
          
          // Save games to storage
          saveGames(games);
          
          // Log the updated state
          logGamesState();
          
          // Get updated games list
          const updatedGames = getAvailableGames();
          
          // Broadcast updated game list to all clients
          console.log(`Broadcasting updated game list after white player reassignment: ${updatedGames.length}`);
          io.emit('availableGames', updatedGames);
          return;
        }
        
        // Join the socket to the game room
        socket.join(gameId);
        
        // Assign as black player
        game.black = socket.id;
        console.log(`Player ${socket.id} joined game ${gameId} as black`);
        
        // Save games to storage
        saveGames(games);
        
        // Notify the joining player they're black
        socket.emit('playerAssigned', { color: 'black' });
        
        // Start the game for both players
        console.log(`Starting game ${gameId} for white player ${game.white} and black player ${socket.id}`);
        io.to(game.white).emit('gameStart', { opponentId: socket.id });
        socket.emit('gameStart', { opponentId: game.white });
        
        // Log the updated state
        logGamesState();
        
        // Get updated games list
        const updatedGames = getAvailableGames();
        
        // Broadcast updated game list to all clients
        console.log(`Broadcasting updated game list after join: ${updatedGames.length}`);
        io.emit('availableGames', updatedGames);
      } catch (error) {
        console.error('Error joining game:', error);
        socket.emit('error', 'Failed to join game');
      }
    });

    // Handle moves
    socket.on('move', ({ gameId, move }) => {
      const game = games.get(gameId);
      if (!game) return;
      
      // Determine which player made the move
      const isWhite = game.white === socket.id;
      const isBlack = game.black === socket.id;
      
      if (!isWhite && !isBlack) return; // Not a player in this game
      
      // Forward the move to the opponent
      const opponent = isWhite ? game.black : game.white;
      io.to(opponent).emit('opponentMove', move);
    });

    // Handle client requesting available games explicitly
    socket.on('getAvailableGames', () => {
      console.log(`Client ${socket.id} requested available games`);
      const availableGames = getAvailableGames();
      socket.emit('availableGames', availableGames);
    });
    
    // Handle adding a test game (for debugging)
    socket.on('addTestGame', () => {
      console.log(`Client ${socket.id} requested to add a test game`);
      addTestGame(io, socket);
    });
    
    // Handle clearing all games (for testing/admin purposes)
    socket.on('clearAllGames', () => {
      console.log(`Client ${socket.id} requested to clear all games`);
      clearAllGames(io, socket);
    });

    // Handle player checking if they're in a game
    socket.on('checkGameStatus', (gameId) => {
      const game = games.get(gameId);
      
      if (!game) {
        socket.emit('gameStatus', { exists: false });
        return;
      }
      
      const isWhitePlayer = game.white === socket.id;
      const isBlackPlayer = game.black === socket.id;
      const gameStarted = game.white && game.black;
      
      // Update this player's connection status in the active sockets map
      if (activeSockets.has(socket.id)) {
        const socketInfo = activeSockets.get(socket.id);
        socketInfo.connected = true;
        socketInfo.lastActive = new Date();
        activeSockets.set(socket.id, socketInfo);
      } else {
        activeSockets.set(socket.id, { 
          connected: true, 
          timestamp: new Date(),
          lastActive: new Date()
        });
      }
      
      // Check if the opponent is connected
      let whiteConnected = false;
      let blackConnected = false;
      
      if (game.white) {
        // If this is the white player, we know they're connected
        if (isWhitePlayer) {
          whiteConnected = true;
        } else {
          // Otherwise check the active sockets map
          whiteConnected = activeSockets.has(game.white) && activeSockets.get(game.white).connected;
        }
      }
      
      if (game.black) {
        // If this is the black player, we know they're connected
        if (isBlackPlayer) {
          blackConnected = true;
        } else {
          // Otherwise check the active sockets map
          blackConnected = activeSockets.has(game.black) && activeSockets.get(game.black).connected;
        }
      }
      
      // If this player is checking status, notify the opponent that this player is connected
      if (isWhitePlayer && game.black && activeSockets.has(game.black) && activeSockets.get(game.black).connected) {
        console.log(`Notifying black player ${game.black} that white player ${socket.id} is connected`);
        io.to(game.black).emit('opponentReconnected');
      } else if (isBlackPlayer && game.white && activeSockets.has(game.white) && activeSockets.get(game.white).connected) {
        console.log(`Notifying white player ${game.white} that black player ${socket.id} is connected`);
        io.to(game.white).emit('opponentReconnected');
      }
      
      // Log the connection status
      console.log(`Game ${gameId} status check: White connected: ${whiteConnected}, Black connected: ${blackConnected}`);
      
      socket.emit('gameStatus', {
        exists: true,
        isWhitePlayer,
        isBlackPlayer,
        gameStarted,
        whiteConnected,
        blackConnected
      });
    });

    socket.on('disconnect', () => {
      console.log('Client disconnected:', socket.id);
      
      // Update active sockets tracking
      if (activeSockets.has(socket.id)) {
        const socketInfo = activeSockets.get(socket.id);
        socketInfo.connected = false;
        socketInfo.disconnectedAt = new Date();
        activeSockets.set(socket.id, socketInfo);
      }
      
      // Find and clean up any games this player was in
      let gamesChanged = false;
      
      games.forEach((game, gameId) => {
        if (game.white === socket.id || game.black === socket.id) {
          // Notify other player if they exist
          const otherPlayer = game.white === socket.id ? game.black : game.white;
          if (otherPlayer && activeSockets.has(otherPlayer) && activeSockets.get(otherPlayer).connected) {
            console.log(`Notifying player ${otherPlayer} that their opponent disconnected`);
            io.to(otherPlayer).emit('opponentDisconnected');
          }
          
          // If this was the white player (creator), mark the game as creator disconnected
          if (game.white === socket.id) {
            console.log(`Creator disconnected from game ${gameId}, marking as available`);
            game.creatorDisconnected = true;
            game.creatorConnected = false;
            gamesChanged = true;
          } 
          // If this was the black player, make the game available again
          else if (game.black === socket.id) {
            console.log(`Making game ${gameId} available again because black player disconnected`);
            game.black = null;
            gamesChanged = true;
          }
        }
      });
      
      // Only save and broadcast updates if games changed
      if (gamesChanged) {
        saveGames(games);
        logGamesState();
        const updatedGames = getAvailableGames();
        console.log(`Broadcasting updated game list after disconnect: ${updatedGames.length}`);
        io.emit('availableGames', updatedGames);
      }
    });

    // Handle reconnection to a game
    socket.on('reconnectToGame', (gameId) => {
      console.log(`Client ${socket.id} attempting to reconnect to game ${gameId}`);
      const game = games.get(gameId);
      
      if (!game) {
        console.log(`Game ${gameId} not found for reconnection`);
        socket.emit('error', 'Game not found');
        return;
      }
      
      // Join the socket to the game room
      socket.join(gameId);
      
      // Update the active sockets map
      activeSockets.set(socket.id, { 
        connected: true, 
        timestamp: new Date(),
        lastActive: new Date()
      });
      
      // Determine if this player is white or black
      const isWhite = game.white === socket.id;
      const isBlack = game.black === socket.id;
      
      // Update the player's socket ID
      if (game.creatorDisconnected) {
        // If the creator was disconnected, this is a new connection as white
        game.white = socket.id;
        game.creatorDisconnected = false;
        game.creatorConnected = true;
        socket.emit('playerAssigned', { color: 'white' });
        console.log(`Player ${socket.id} reconnected as white to game ${gameId}`);
      } else if (isWhite || (game.white && !activeSockets.get(game.white)?.connected)) {
        // This is the white player reconnecting
        if (!isWhite) {
          game.white = socket.id;
        }
        game.creatorDisconnected = false;
        game.creatorConnected = true;
        socket.emit('playerAssigned', { color: 'white' });
        console.log(`Player ${socket.id} reconnected as white to game ${gameId}`);
        
        // If black player exists, notify them
        if (game.black && activeSockets.has(game.black) && activeSockets.get(game.black).connected) {
          console.log(`Notifying black player ${game.black} that white has reconnected`);
          io.to(game.black).emit('opponentReconnected');
        }
      } else if (isBlack || (game.black && !activeSockets.get(game.black)?.connected)) {
        // This is the black player reconnecting
        if (!isBlack) {
          game.black = socket.id;
        }
        socket.emit('playerAssigned', { color: 'black' });
        console.log(`Player ${socket.id} reconnected as black to game ${gameId}`);
        
        // If white player exists, notify them
        if (game.white && activeSockets.has(game.white) && activeSockets.get(game.white).connected) {
          console.log(`Notifying white player ${game.white} that black has reconnected`);
          io.to(game.white).emit('opponentReconnected');
        }
      } else if (!game.black) {
        // This is a new player joining as black
        game.black = socket.id;
        socket.emit('playerAssigned', { color: 'black' });
        console.log(`Player ${socket.id} joined as black to game ${gameId}`);
        
        // If white player exists, notify them
        if (game.white && activeSockets.has(game.white) && activeSockets.get(game.white).connected) {
          console.log(`Notifying white player ${game.white} that black has joined`);
          io.to(game.white).emit('opponentReconnected');
        }
        
        // Start the game
        io.to(game.white).emit('gameStart', { opponentId: socket.id });
        socket.emit('gameStart', { opponentId: game.white });
      }
      
      // Save games to storage
      saveGames(games);
      
      // Log the updated state
      logGamesState();
    });
  });
}

// Export both the initSocket function and the games Map for testing
module.exports = { initSocket, games }; 
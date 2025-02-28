const fs = require('fs');
const path = require('path');

// Path to the games storage file
const STORAGE_FILE = path.join(__dirname, 'games.json');

// Initialize the storage file if it doesn't exist
function initStorage() {
  if (!fs.existsSync(STORAGE_FILE)) {
    fs.writeFileSync(STORAGE_FILE, JSON.stringify({}));
    console.log('Created new games storage file');
  }
}

// Load games from storage
function loadGames() {
  try {
    initStorage();
    const data = fs.readFileSync(STORAGE_FILE, 'utf8');
    const games = JSON.parse(data);
    
    // Convert the plain object back to a Map
    const gamesMap = new Map();
    Object.entries(games).forEach(([gameId, game]) => {
      // Convert createdAt string back to Date object
      if (game.createdAt) {
        game.createdAt = new Date(game.createdAt);
      }
      gamesMap.set(gameId, game);
    });
    
    console.log(`Loaded ${gamesMap.size} games from storage`);
    return gamesMap;
  } catch (error) {
    console.error('Error loading games from storage:', error);
    return new Map();
  }
}

// Save games to storage
function saveGames(gamesMap) {
  try {
    // Convert the Map to a plain object for JSON serialization
    const games = {};
    gamesMap.forEach((game, gameId) => {
      games[gameId] = game;
    });
    
    fs.writeFileSync(STORAGE_FILE, JSON.stringify(games, null, 2));
    console.log(`Saved ${gamesMap.size} games to storage`);
  } catch (error) {
    console.error('Error saving games to storage:', error);
  }
}

module.exports = {
  loadGames,
  saveGames
}; 
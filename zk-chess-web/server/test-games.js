/**
 * This is a simple test script to verify that the server is storing games correctly.
 * Run it with: node server/test-games.js
 */

const { games } = require('./socket');

// Create a test game
const gameId = 'test-' + Math.random().toString(36).substring(2, 8);
games.set(gameId, {
  white: 'test-player-white',
  black: null,
  createdAt: new Date()
});

console.log('===== TEST GAME CREATED =====');
console.log(`Game ID: ${gameId}`);
console.log('This game should appear in the available games list.');
console.log('If it does not, there may be an issue with how games are stored or retrieved.');
console.log('');
console.log('Current games in memory:');

// Log all games
games.forEach((game, id) => {
  console.log(`Game ${id}:`);
  console.log(`  White player: ${game.white}`);
  console.log(`  Black player: ${game.black || 'Not joined'}`);
  console.log(`  Created at: ${game.createdAt}`);
  console.log(`  Available: ${!game.black ? 'Yes' : 'No'}`);
});

console.log('');
console.log('If you see the test game above, the games Map is working correctly.');
console.log('If you do not see the test game, there may be an issue with how the games Map is shared between modules.');
console.log('');
console.log('To test if the game appears in the UI:');
console.log('1. Start the server: npm run dev');
console.log('2. Open the browser to http://localhost:3000');
console.log('3. Check if the test game appears in the available games list.');
console.log('');
console.log('If the test game does not appear, the issue may be with:');
console.log('- How the server sends available games to clients');
console.log('- How the client receives and displays available games');
console.log('');
console.log('You can run this script again to add more test games.'); 
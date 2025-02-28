import { useState } from 'react';
import io from 'socket.io-client';
import styles from '../styles/ClearGames.module.css';

export default function ClearGames() {
  const [isClearing, setIsClearing] = useState(false);
  const [result, setResult] = useState(null);

  const clearAllGames = async () => {
    setIsClearing(true);
    setResult(null);
    
    try {
      // Create a temporary socket just for this operation
      const socket = io();
      
      // Wait for connection
      await new Promise((resolve) => {
        socket.on('connect', resolve);
        // Set a timeout in case connection fails
        setTimeout(resolve, 2000);
      });
      
      // Request to clear all games
      socket.emit('clearAllGames');
      
      // Wait for confirmation
      const response = await new Promise((resolve) => {
        socket.on('gamesCleared', (data) => resolve(data));
        // Set a timeout in case response never comes
        setTimeout(() => resolve({ success: false, error: 'Timeout waiting for response' }), 3000);
      });
      
      if (response.success) {
        setResult({ success: true, message: `Cleared ${response.count} games` });
      } else {
        setResult({ success: false, message: response.error || 'Failed to clear games' });
      }
      
      // Disconnect the socket
      socket.disconnect();
    } catch (error) {
      console.error('Error clearing games:', error);
      setResult({ success: false, message: error.message || 'An error occurred' });
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <div className={styles.container}>
      <h2>Admin Tools</h2>
      <button 
        className={styles.clearButton}
        onClick={clearAllGames}
        disabled={isClearing}
      >
        {isClearing ? 'Clearing...' : 'Clear All Games'}
      </button>
      
      {result && (
        <div className={result.success ? styles.success : styles.error}>
          {result.message}
        </div>
      )}
    </div>
  );
} 
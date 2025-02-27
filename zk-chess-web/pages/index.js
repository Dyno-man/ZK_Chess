import { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/router';
import io from 'socket.io-client';
import styles from '../styles/Home.module.css';
import ClearGames from '../components/ClearGames';

// Global socket instance that persists across page navigations
let globalSocket = null;

export default function Home() {
  const router = useRouter();
  const [error, setError] = useState(null);
  const [socket, setSocket] = useState(null);
  const [availableGames, setAvailableGames] = useState([]);
  const [joiningGameId, setJoiningGameId] = useState(null);
  const [socketConnected, setSocketConnected] = useState(false);
  const [lastUpdated, setLastUpdated] = useState(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isAddingTestGame, setIsAddingTestGame] = useState(false);
  const [isCreatingGame, setIsCreatingGame] = useState(false);
  const [showAdminTools, setShowAdminTools] = useState(false);
  
  // Use a ref to track if the component is mounted
  const isMounted = useRef(true);
  // Use a ref to track the socket instance
  const socketRef = useRef(null);
  
  useEffect(() => {
    // Set up cleanup function
    return () => {
      isMounted.current = false;
    };
  }, []);
  
  useEffect(() => {
    console.log('Home page loaded');
    
    // Clear any previous game data
    localStorage.removeItem('currentGameId');
    localStorage.removeItem('playerColor');
    
    // Create a new socket connection or use the existing global one
    let newSocket;
    if (globalSocket && globalSocket.connected) {
      console.log('Using existing socket connection');
      newSocket = globalSocket;
    } else {
      console.log('Creating new socket connection');
      newSocket = io();
      globalSocket = newSocket; // Store in global variable
    }
    
    setSocket(newSocket);
    socketRef.current = newSocket;
    
    // If socket is already connected, update the state
    if (newSocket.connected) {
      console.log('Socket already connected with ID:', newSocket.id);
      setSocketConnected(true);
      setError(null);
      
      // Request available games
      console.log('Requesting available games for already connected socket');
      newSocket.emit('getAvailableGames');
    }
    
    // Socket connection events
    newSocket.on('connect', () => {
      console.log('Socket connected with ID:', newSocket.id);
      setSocketConnected(true);
      setError(null); // Clear any previous connection errors
      
      // Explicitly request available games when socket connects
      console.log('Requesting available games after connection');
      newSocket.emit('getAvailableGames');
    });
    
    newSocket.on('disconnect', () => {
      console.log('Socket disconnected');
      setSocketConnected(false);
      setError('Disconnected from server. Trying to reconnect...');
    });
    
    // When a game is created, store the ID and navigate
    newSocket.on('gameCreated', (data) => {
      const createdGameId = data.gameId;
      console.log('Game created with ID:', createdGameId);
      setIsCreatingGame(false); // Reset creating state
      localStorage.setItem('currentGameId', createdGameId);
      
      // Don't disconnect the socket when navigating
      router.push(`/game/${createdGameId}`);
    });
    
    newSocket.on('error', (message) => {
      console.error('Socket error:', message);
      setError(message);
      setJoiningGameId(null); // Reset joining state on error
      setIsAddingTestGame(false); // Reset test game state on error
      setIsRefreshing(false); // Reset refreshing state on error
      setIsCreatingGame(false); // Reset creating state on error
    });

    newSocket.on('availableGames', (games) => {
      console.log('Received available games:', games);
      if (isMounted.current) {
        setAvailableGames(games || []);
        setLastUpdated(new Date());
        setIsRefreshing(false); // Reset refreshing state
      }
    });
    
    // Handle test game added confirmation
    newSocket.on('testGameAdded', ({ gameId }) => {
      console.log('Test game added with ID:', gameId);
      if (isMounted.current) {
        setIsAddingTestGame(false);
      }
      // The availableGames event will be triggered separately
    });

    // Handle player assignment
    newSocket.on('playerAssigned', ({ color }) => {
      console.log('Player assigned as:', color);
      localStorage.setItem('playerColor', color);
      
      // If we're joining a game, navigate to the game page
      if (joiningGameId) {
        localStorage.setItem('currentGameId', joiningGameId);
        router.push(`/game/${joiningGameId}`);
      }
    });
    
    // Set up a periodic refresh of available games
    const refreshInterval = setInterval(() => {
      if (newSocket && newSocket.connected) {
        console.log('Requesting available games refresh');
        newSocket.emit('getAvailableGames');
      }
    }, 10000); // Refresh every 10 seconds
    
    return () => {
      console.log('Cleaning up Home component');
      clearInterval(refreshInterval);
      
      // Don't disconnect the socket when navigating to game page
      // We'll keep it in the global variable for reuse
      // Only remove event listeners to prevent memory leaks
      newSocket.off('connect');
      newSocket.off('disconnect');
      newSocket.off('gameCreated');
      newSocket.off('error');
      newSocket.off('availableGames');
      newSocket.off('testGameAdded');
      newSocket.off('playerAssigned');
    };
  }, [router, joiningGameId]);
  
  const createGame = () => {
    console.log('Creating new game');
    if (socketRef.current && socketConnected) {
      setIsCreatingGame(true);
      socketRef.current.emit('createGame');
      
      // Set a timeout to reset the creating state if no response is received
      setTimeout(() => {
        if (isMounted.current && isCreatingGame) {
          console.log('Create game timeout - resetting state');
          setIsCreatingGame(false);
          setError('Game creation timed out. Please try again.');
        }
      }, 3000);
    } else {
      console.error('Socket not initialized or not connected');
      setError('Not connected to server. Please refresh the page.');
    }
  };
  
  const joinGame = (gameIdToJoin) => {
    console.log('Joining game:', gameIdToJoin);
    if (socketRef.current && socketConnected) {
      setJoiningGameId(gameIdToJoin);
      localStorage.setItem('currentGameId', gameIdToJoin);
      socketRef.current.emit('joinGame', gameIdToJoin);
      
      // Set a timeout to reset the joining state if no response is received
      setTimeout(() => {
        if (isMounted.current && joiningGameId === gameIdToJoin) {
          console.log('Join game timeout - resetting state');
          setJoiningGameId(null);
          setError('Game joining timed out. Please try again.');
        }
      }, 3000);
    } else {
      console.error('Socket not initialized or not connected');
      setError('Not connected to server. Please refresh the page.');
    }
  };
  
  // Manual refresh button handler
  const refreshGames = () => {
    console.log('Manually refreshing games');
    if (socketRef.current && socketConnected) {
      setIsRefreshing(true);
      socketRef.current.emit('getAvailableGames');
      
      // Set a timeout to reset the refreshing state if no response is received
      setTimeout(() => {
        if (isMounted.current && isRefreshing) {
          setIsRefreshing(false);
          setError('Refresh timed out. Please try again.');
        }
      }, 3000);
    } else {
      console.error('Socket not initialized or not connected');
      setError('Not connected to server. Please refresh the page.');
    }
  };
  
  // Add a test game for debugging
  const addTestGame = () => {
    console.log('Adding test game');
    if (socketRef.current && socketConnected) {
      setIsAddingTestGame(true);
      socketRef.current.emit('addTestGame');
      
      // Set a timeout to reset the adding state if no response is received
      setTimeout(() => {
        if (isMounted.current && isAddingTestGame) {
          console.log('Test game timeout - resetting state');
          setIsAddingTestGame(false);
          setError('Adding test game timed out. Please try again.');
          // Try to refresh games list
          socketRef.current.emit('getAvailableGames');
        }
      }, 3000);
    } else {
      console.error('Socket not initialized or not connected');
      setError('Not connected to server. Please refresh the page.');
    }
  };
  
  // Toggle admin tools
  const toggleAdminTools = () => {
    setShowAdminTools(!showAdminTools);
  };
  
  // Clear error message
  const clearError = () => {
    setError(null);
  };
  
  return (
    <div className={styles.container}>
      <h1>ZK Chess</h1>
      <p>Play chess with zero-knowledge proof verification</p>
      
      {error && (
        <div className={styles.error}>
          {error}
          <button onClick={clearError} className={styles.closeError}>×</button>
        </div>
      )}
      
      <div className={styles.actions}>
        <button 
          onClick={createGame} 
          className={styles.button}
          disabled={isCreatingGame || !socketConnected}
        >
          {isCreatingGame ? 'Creating Game...' : 'Create New Game'}
        </button>
        
        <div className={styles.availableGames}>
          <div className={styles.gamesHeader}>
            <h2>Available Games</h2>
            <button 
              onClick={refreshGames} 
              className={styles.refreshButton}
              disabled={isRefreshing || !socketConnected}
            >
              {isRefreshing ? 'Refreshing...' : 'Refresh'}
            </button>
          </div>
          
          {availableGames.length === 0 ? (
            <div className={styles.noGamesContainer}>
              <p className={styles.noGames}>No games available. Create one to start playing!</p>
              <button 
                onClick={addTestGame} 
                className={styles.createTestButton}
                disabled={isAddingTestGame || !socketConnected}
              >
                {isAddingTestGame ? 'Creating Test Game...' : 'Create Test Game'}
              </button>
            </div>
          ) : (
            <ul className={styles.gamesList}>
              {availableGames.map((game) => (
                <li key={game.gameId} className={styles.gameItem}>
                  <div className={styles.gameInfo}>
                    <span className={styles.gameId}>Game: {game.gameId}</span>
                    <span className={styles.gameTime}>Created: {new Date(game.createdAt).toLocaleTimeString()}</span>
                  </div>
                  <button 
                    onClick={() => joinGame(game.gameId)}
                    className={styles.joinButton}
                    disabled={joiningGameId === game.gameId || !socketConnected}
                  >
                    {joiningGameId === game.gameId ? 'Joining...' : 'Join Game'}
                  </button>
                </li>
              ))}
            </ul>
          )}
          
          {lastUpdated && (
            <div className={styles.lastUpdated}>
              Last updated: {lastUpdated.toLocaleTimeString()}
              {isRefreshing && <span className={styles.refreshingIndicator}> (refreshing...)</span>}
            </div>
          )}
        </div>
      </div>
      
      <div className={styles.debug}>
        <p>Debug Info:</p>
        <p>Socket connected: {socketConnected ? 'Yes' : 'No'}</p>
        <p>Socket ID: {socket ? socket.id : 'Not connected'}</p>
        <p>Available games: {availableGames.length}</p>
        <p>Last updated: {lastUpdated ? lastUpdated.toLocaleTimeString() : 'Never'}</p>
        <button 
          onClick={addTestGame} 
          className={styles.debugButton}
          disabled={isAddingTestGame || !socketConnected}
        >
          {isAddingTestGame ? 'Adding Test Game...' : 'Add Test Game'}
        </button>
        
        <button 
          onClick={toggleAdminTools} 
          className={styles.debugButton}
          style={{ marginLeft: '10px' }}
        >
          {showAdminTools ? 'Hide Admin Tools' : 'Show Admin Tools'}
        </button>
        
        {showAdminTools && <ClearGames />}
        
        <pre>{JSON.stringify(availableGames, null, 2)}</pre>
      </div>
    </div>
  );
} 
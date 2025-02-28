import { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/router';
import io from 'socket.io-client';
import styles from '../../styles/Game.module.css';
import ChessBoard from '../../components/ChessBoard';

// Use the same global socket instance from index.js
// This is declared as external to avoid redeclaring it
// The actual instance is created in index.js
let globalSocket = null;

export default function Game() {
  const router = useRouter();
  const { gameId } = router.query;
  const [socket, setSocket] = useState(null);
  const [playerColor, setPlayerColor] = useState(null);
  const [gameStarted, setGameStarted] = useState(false);
  const [error, setError] = useState(null);
  const [moves, setMoves] = useState([]);
  const [socketConnected, setSocketConnected] = useState(false);
  const [opponentId, setOpponentId] = useState(null);
  const [opponentConnected, setOpponentConnected] = useState(false);
  const [isCheckingStatus, setIsCheckingStatus] = useState(false);
  const [lastStatusCheck, setLastStatusCheck] = useState(null);
  
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
    if (!gameId) return; // Wait for gameId to be available
    
    console.log('Game page loaded for game:', gameId);
    
    // Get player color from localStorage
    const storedColor = localStorage.getItem('playerColor');
    if (storedColor) {
      console.log('Player color from localStorage:', storedColor);
      setPlayerColor(storedColor);
    }
    
    // Use the existing global socket or create a new one
    let newSocket;
    if (globalSocket && globalSocket.connected) {
      console.log('Using existing socket connection in game page');
      newSocket = globalSocket;
    } else {
      console.log('Creating new socket connection in game page');
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
      
      // Attempt to reconnect to the game
      console.log('Attempting to reconnect to game:', gameId);
      newSocket.emit('reconnectToGame', gameId);
      
      // Check game status
      checkGameStatus(newSocket, gameId);
    }
    
    // Socket connection events
    newSocket.on('connect', () => {
      console.log('Socket connected in game page with ID:', newSocket.id);
      setSocketConnected(true);
      setError(null);
      
      // Attempt to reconnect to the game
      console.log('Attempting to reconnect to game after connection:', gameId);
      newSocket.emit('reconnectToGame', gameId);
      
      // Check game status
      checkGameStatus(newSocket, gameId);
    });
    
    newSocket.on('disconnect', () => {
      console.log('Socket disconnected in game page');
      setSocketConnected(false);
      setOpponentConnected(false);
      setError('Disconnected from server. Game state may be lost.');
    });
    
    // Handle player assignment
    newSocket.on('playerAssigned', ({ color }) => {
      console.log('Player assigned as:', color);
      setPlayerColor(color);
      localStorage.setItem('playerColor', color);
    });
    
    // Handle game start
    newSocket.on('gameStart', (data) => {
      console.log('Game started with opponent:', data?.opponentId);
      setGameStarted(true);
      if (data?.opponentId) {
        setOpponentId(data.opponentId);
        setOpponentConnected(true);
      }
    });
    
    // Handle opponent moves
    newSocket.on('opponentMove', (move) => {
      console.log('Opponent made move:', move);
      setMoves((prevMoves) => [...prevMoves, move]);
    });
    
    // Handle opponent disconnection
    newSocket.on('opponentDisconnected', () => {
      console.log('Opponent disconnected');
      setOpponentConnected(false);
      setError('Your opponent has disconnected. You can continue waiting or return to the lobby.');
    });
    
    // Handle opponent reconnection
    newSocket.on('opponentReconnected', () => {
      console.log('Opponent reconnected');
      setOpponentConnected(true);
      setError(null);
    });
    
    // Handle game status response
    newSocket.on('gameStatus', (status) => {
      console.log('Received game status:', status);
      setIsCheckingStatus(false);
      setLastStatusCheck(new Date());
      
      if (status.exists) {
        if (status.isWhitePlayer) {
          setPlayerColor('white');
        } else if (status.isBlackPlayer) {
          setPlayerColor('black');
        }
        
        if (status.gameStarted) {
          setGameStarted(true);
        }
        
        // Update opponent connection status
        if (status.isWhitePlayer && status.blackConnected) {
          setOpponentConnected(true);
        } else if (status.isBlackPlayer && status.whiteConnected) {
          setOpponentConnected(true);
        } else {
          setOpponentConnected(false);
        }
      }
    });
    
    // Set up periodic game status check
    const statusInterval = setInterval(() => {
      if (newSocket && newSocket.connected && gameId) {
        checkGameStatus(newSocket, gameId);
      }
    }, 5000); // Check every 5 seconds
    
    return () => {
      console.log('Cleaning up Game component');
      clearInterval(statusInterval);
      
      // Don't disconnect the socket, just remove event listeners
      newSocket.off('connect');
      newSocket.off('disconnect');
      newSocket.off('playerAssigned');
      newSocket.off('gameStart');
      newSocket.off('opponentMove');
      newSocket.off('opponentDisconnected');
      newSocket.off('opponentReconnected');
      newSocket.off('gameStatus');
    };
  }, [gameId]);
  
  // Helper function to check game status
  const checkGameStatus = (socket, gameId) => {
    if (!socket || !gameId) return;
    
    console.log('Checking game status for game:', gameId);
    setIsCheckingStatus(true);
    socket.emit('checkGameStatus', gameId);
    
    // Set a timeout to reset the checking state if no response
    setTimeout(() => {
      if (isMounted.current && isCheckingStatus) {
        setIsCheckingStatus(false);
      }
    }, 3000);
  };
  
  // Function to manually check game status
  const refreshStatus = () => {
    if (socketRef.current && socketConnected && gameId) {
      checkGameStatus(socketRef.current, gameId);
    }
  };
  
  const makeMove = (move) => {
    if (socketRef.current && socketConnected && gameId) {
      console.log('Making move:', move);
      socketRef.current.emit('move', { gameId, move });
      setMoves((prevMoves) => [...prevMoves, move]);
    } else {
      console.error('Cannot make move: socket not connected');
      setError('Not connected to server. Please refresh the page.');
    }
  };
  
  const returnToLobby = () => {
    router.push('/');
  };
  
  // Clear error message
  const clearError = () => {
    setError(null);
  };
  
  if (!gameId) {
    return <div className={styles.loading}>Loading game...</div>;
  }
  
  return (
    <div className={styles.container}>
      <h1>Game: {gameId}</h1>
      
      {error && (
        <div className={styles.error}>
          {error}
          <button onClick={clearError} className={styles.closeError}>×</button>
        </div>
      )}
      
      <div className={styles.gameInfo}>
        <p>You are playing as: <span className={styles.highlight}>{playerColor || 'Not assigned yet'}</span></p>
        
        <div className={styles.statusContainer}>
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Game status:</span>
            <span className={gameStarted ? styles.statusGood : styles.statusWaiting}>
              {gameStarted ? 'Game in progress' : 'Waiting to start'}
            </span>
          </div>
          
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Your connection:</span>
            <span className={socketConnected ? styles.statusGood : styles.statusBad}>
              {socketConnected ? 'Connected' : 'Disconnected'}
            </span>
            {!socketConnected && (
              <button 
                onClick={() => window.location.reload()} 
                className={styles.reconnectButton}
              >
                Reconnect
              </button>
            )}
          </div>
          
          <div className={styles.statusItem}>
            <span className={styles.statusLabel}>Opponent:</span>
            <span className={opponentConnected ? styles.statusGood : styles.statusBad}>
              {opponentConnected ? 'Connected' : 'Disconnected or waiting'}
            </span>
            <button 
              onClick={refreshStatus} 
              className={styles.refreshButton}
              disabled={isCheckingStatus}
            >
              {isCheckingStatus ? 'Checking...' : 'Check'}
            </button>
          </div>
        </div>
        
        {lastStatusCheck && (
          <p className={styles.lastChecked}>
            Last status check: {lastStatusCheck.toLocaleTimeString()}
            {isCheckingStatus && <span className={styles.refreshingIndicator}> (checking...)</span>}
          </p>
        )}
      </div>
      
      <div className={styles.boardContainer}>
        {playerColor && gameStarted ? (
          <ChessBoard playerColor={playerColor} onMove={makeMove} moves={moves} />
        ) : (
          <div className={styles.waitingMessage}>
            {!playerColor ? 'Waiting for player assignment...' : 'Waiting for opponent to connect...'}
          </div>
        )}
      </div>
      
      <button onClick={returnToLobby} className={styles.returnButton}>
        Return to Lobby
      </button>
    </div>
  );
} 
import { useState, useEffect } from 'react';
import styles from '../styles/ChessBoard.module.css';

// Initial position setup
const initialPosition = {
  a1: { type: 'rook', color: 'white' },
  b1: { type: 'knight', color: 'white' },
  c1: { type: 'bishop', color: 'white' },
  d1: { type: 'queen', color: 'white' },
  e1: { type: 'king', color: 'white' },
  f1: { type: 'bishop', color: 'white' },
  g1: { type: 'knight', color: 'white' },
  h1: { type: 'rook', color: 'white' },
  a2: { type: 'pawn', color: 'white' },
  b2: { type: 'pawn', color: 'white' },
  c2: { type: 'pawn', color: 'white' },
  d2: { type: 'pawn', color: 'white' },
  e2: { type: 'pawn', color: 'white' },
  f2: { type: 'pawn', color: 'white' },
  g2: { type: 'pawn', color: 'white' },
  h2: { type: 'pawn', color: 'white' },
  
  a8: { type: 'rook', color: 'black' },
  b8: { type: 'knight', color: 'black' },
  c8: { type: 'bishop', color: 'black' },
  d8: { type: 'queen', color: 'black' },
  e8: { type: 'king', color: 'black' },
  f8: { type: 'bishop', color: 'black' },
  g8: { type: 'knight', color: 'black' },
  h8: { type: 'rook', color: 'black' },
  a7: { type: 'pawn', color: 'black' },
  b7: { type: 'pawn', color: 'black' },
  c7: { type: 'pawn', color: 'black' },
  d7: { type: 'pawn', color: 'black' },
  e7: { type: 'pawn', color: 'black' },
  f7: { type: 'pawn', color: 'black' },
  g7: { type: 'pawn', color: 'black' },
  h7: { type: 'pawn', color: 'black' },
};

// Unicode chess symbols
const pieceSymbols = {
  king: { white: '♔', black: '♚' },
  queen: { white: '♕', black: '♛' },
  rook: { white: '♖', black: '♜' },
  bishop: { white: '♗', black: '♝' },
  knight: { white: '♘', black: '♞' },
  pawn: { white: '♙', black: '♟' },
};

export default function ChessBoard({ playerColor = 'white', onMove, lastMove, gameStarted = false }) {
  const [position, setPosition] = useState(initialPosition);
  const [selectedSquare, setSelectedSquare] = useState(null);
  const [validMoves, setValidMoves] = useState([]);
  const [currentTurn, setCurrentTurn] = useState('white'); // White always starts
  
  // Apply opponent's move
  useEffect(() => {
    if (lastMove && lastMove.from && lastMove.to) {
      makeMove(lastMove.from, lastMove.to, true);
    }
  }, [lastMove]);
  
  // Get the symbol for a piece
  const getPieceSymbol = (type, color) => {
    if (!type || !color) return '';
    return pieceSymbols[type][color];
  };
  
  // Check if it's the player's turn
  const isPlayerTurn = () => {
    return playerColor === currentTurn;
  };
  
  // Get valid moves for a piece
  const getValidMoves = (square) => {
    const piece = position[square];
    if (!piece) return [];
    
    // Simple implementation - in a real app, you'd implement proper chess rules
    const moves = [];
    const [file, rank] = [square.charCodeAt(0) - 97, parseInt(square[1])];
    
    // Pawns
    if (piece.type === 'pawn') {
      const direction = piece.color === 'white' ? 1 : -1;
      const newRank = rank + direction;
      
      // Forward move
      if (newRank >= 1 && newRank <= 8) {
        const newSquare = `${String.fromCharCode(file + 97)}${newRank}`;
        if (!position[newSquare]) {
          moves.push(newSquare);
          
          // Double move from starting position
          if ((piece.color === 'white' && rank === 2) || (piece.color === 'black' && rank === 7)) {
            const doubleRank = rank + 2 * direction;
            const doubleSquare = `${String.fromCharCode(file + 97)}${doubleRank}`;
            if (!position[doubleSquare]) {
              moves.push(doubleSquare);
            }
          }
        }
      }
      
      // Captures
      for (let offset of [-1, 1]) {
        const newFile = file + offset;
        if (newFile >= 0 && newFile < 8) {
          const captureSquare = `${String.fromCharCode(newFile + 97)}${newRank}`;
          const targetPiece = position[captureSquare];
          if (targetPiece && targetPiece.color !== piece.color) {
            moves.push(captureSquare);
          }
        }
      }
    }
    
    // For simplicity, other pieces can move anywhere that's empty or has an opponent's piece
    // In a real app, you'd implement proper movement rules for each piece type
    else {
      for (let i = 0; i < 64; i++) {
        const f = i % 8;
        const r = Math.floor(i / 8) + 1;
        const sq = `${String.fromCharCode(f + 97)}${r}`;
        
        // Skip the current square and squares with our own pieces
        if (sq === square) continue;
        if (position[sq] && position[sq].color === piece.color) continue;
        
        // For simplicity, allow any move (not realistic chess rules)
        moves.push(sq);
      }
    }
    
    return moves;
  };
  
  // Handle square click
  const handleSquareClick = (square) => {
    if (!gameStarted || !isPlayerTurn()) return;
    
    const piece = position[square];
    
    // If a piece is already selected
    if (selectedSquare) {
      // If clicking on a valid move square, make the move
      if (validMoves.includes(square)) {
        makeMove(selectedSquare, square);
      } 
      // If clicking on another of your pieces, select that piece instead
      else if (piece && piece.color === playerColor) {
        setSelectedSquare(square);
        setValidMoves(getValidMoves(square));
      } 
      // Otherwise, deselect
      else {
        setSelectedSquare(null);
        setValidMoves([]);
      }
    } 
    // If no piece is selected yet, select if it's your piece
    else if (piece && piece.color === playerColor) {
      setSelectedSquare(square);
      setValidMoves(getValidMoves(square));
    }
  };
  
  // Make a move
  const makeMove = (from, to, isOpponentMove = false) => {
    const piece = position[from];
    if (!piece) return;
    
    // Create new position
    const newPosition = { ...position };
    delete newPosition[from];
    newPosition[to] = piece;
    
    // Update state
    setPosition(newPosition);
    setSelectedSquare(null);
    setValidMoves([]);
    setCurrentTurn(currentTurn === 'white' ? 'black' : 'white');
    
    // Notify parent component if this is our move
    if (!isOpponentMove) {
      onMove({ from, to, piece, captured: position[to] });
    }
  };
  
  // Render a square
  const renderSquare = (i) => {
    const file = i % 8;
    const rank = Math.floor(i / 8);
    const square = `${String.fromCharCode(97 + file)}${8 - rank}`;
    const piece = position[square] || null; // Add null fallback if square doesn't exist
    
    return (
      <div
        key={square}
        className={`${styles.square} ${(file + rank) % 2 === 0 ? styles.light : styles.dark} ${
          selectedSquare === square ? styles.selected : ''
        } ${
          validMoves.includes(square) ? styles.validMove : ''
        }`}
        onClick={() => handleSquareClick(square)}
      >
        {piece && (
          <div className={styles.piece}>
            {getPieceSymbol(piece.type, piece.color)}
          </div>
        )}
      </div>
    );
  };
  
  // Render the board
  const renderBoard = () => {
    const squares = [];
    // Flip the board if playing as black
    const isFlipped = playerColor === 'black';
    
    for (let i = 0; i < 64; i++) {
      const index = isFlipped ? 63 - i : i;
      squares.push(renderSquare(index));
    }
    
    return squares;
  };
  
  return (
    <div className={styles.board}>
      {renderBoard()}
      {!gameStarted && (
        <div className={styles.overlay}>
          <p>Waiting for opponent...</p>
        </div>
      )}
      {gameStarted && !isPlayerTurn() && (
        <div className={styles.turnIndicator}>
          Waiting for opponent's move...
        </div>
      )}
    </div>
  );
} 
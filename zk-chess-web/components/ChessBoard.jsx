import { useState, useEffect } from 'react';
import styles from '../styles/ChessBoard.module.css';

export default function ChessBoard({ board, playerColor, onMakeMove, isMyTurn }) {
  const [selectedPiece, setSelectedPiece] = useState(null);
  
  const handleSquareClick = (x, y) => {
    if (!isMyTurn) return;
    
    if (selectedPiece) {
      // Attempt to make a move
      onMakeMove({
        from_x: selectedPiece.x,
        from_y: selectedPiece.y,
        to_x: x,
        to_y: y,
        // Add other required move data
      });
      setSelectedPiece(null);
    } else {
      // Select a piece if it belongs to the player
      const piece = getPieceAt(board, x, y);
      if (piece && isPieceOwnedByPlayer(piece, playerColor)) {
        setSelectedPiece({ x, y, piece });
      }
    }
  };
  
  const renderBoard = () => {
    const squares = [];
    // Flip board for black player
    const range = playerColor === 'black' ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
    
    for (let y of range) {
      for (let x of (playerColor === 'black' ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7])) {
        const isSelected = selectedPiece && selectedPiece.x === x && selectedPiece.y === y;
        const piece = getPieceAt(board, x, y);
        
        squares.push(
          <div 
            key={`${x}-${y}`}
            className={`${styles.square} ${(x + y) % 2 === 0 ? styles.light : styles.dark} ${isSelected ? styles.selected : ''}`}
            onClick={() => handleSquareClick(x, y)}
          >
            {piece && renderPiece(piece)}
          </div>
        );
      }
    }
    return squares;
  };
  
  return (
    <div className={styles.board}>
      {renderBoard()}
    </div>
  );
}

// Helper functions
function getPieceAt(board, x, y) {
  // Extract piece from board representation
  const idx = y * 8 + x;
  return board[idx];
}

function isPieceOwnedByPlayer(piece, playerColor) {
  // Check if piece belongs to player based on color
  const pieceColor = (piece >> 3) & 1;
  return (pieceColor === 0 && playerColor === 'white') || 
         (pieceColor === 1 && playerColor === 'black');
}

function renderPiece(piece) {
  // Map piece code to Unicode chess symbols
  const pieceType = piece & 7;
  const pieceColor = (piece >> 3) & 1;
  const isWhite = pieceColor === 0;
  
  const symbols = {
    0: isWhite ? '♙' : '♟', // Pawn
    1: isWhite ? '♘' : '♞', // Knight
    2: isWhite ? '♗' : '♝', // Bishop
    3: isWhite ? '♖' : '♜', // Rook
    4: isWhite ? '♕' : '♛', // Queen
    5: isWhite ? '♔' : '♚', // King
  };
  
  return <span className={styles.piece}>{symbols[pieceType]}</span>;
} 
import { verifyMove } from './zkProofs';

export class GameState {
  constructor() {
    this.position = this.getInitialPosition();
    this.moveHistory = [];
    this.currentTurn = 'white';
  }

  getInitialPosition() {
    return {
      'a1': { type: 'rook', color: 'white', unicode: '♖' },
      'b1': { type: 'knight', color: 'white', unicode: '♘' },
      'c1': { type: 'bishop', color: 'white', unicode: '♗' },
      'd1': { type: 'queen', color: 'white', unicode: '♕' },
      'e1': { type: 'king', color: 'white', unicode: '♔' },
      'f1': { type: 'bishop', color: 'white', unicode: '♗' },
      'g1': { type: 'knight', color: 'white', unicode: '♘' },
      'h1': { type: 'rook', color: 'white', unicode: '♖' },
      'a2': { type: 'pawn', color: 'white', unicode: '♙' },
      'b2': { type: 'pawn', color: 'white', unicode: '♙' },
      'c2': { type: 'pawn', color: 'white', unicode: '♙' },
      'd2': { type: 'pawn', color: 'white', unicode: '♙' },
      'e2': { type: 'pawn', color: 'white', unicode: '♙' },
      'f2': { type: 'pawn', color: 'white', unicode: '♙' },
      'g2': { type: 'pawn', color: 'white', unicode: '♙' },
      'h2': { type: 'pawn', color: 'white', unicode: '♙' },
      
      'a8': { type: 'rook', color: 'black', unicode: '♜' },
      'b8': { type: 'knight', color: 'black', unicode: '♞' },
      'c8': { type: 'bishop', color: 'black', unicode: '♝' },
      'd8': { type: 'queen', color: 'black', unicode: '♛' },
      'e8': { type: 'king', color: 'black', unicode: '♚' },
      'f8': { type: 'bishop', color: 'black', unicode: '♝' },
      'g8': { type: 'knight', color: 'black', unicode: '♞' },
      'h8': { type: 'rook', color: 'black', unicode: '♜' },
      'a7': { type: 'pawn', color: 'black', unicode: '♟' },
      'b7': { type: 'pawn', color: 'black', unicode: '♟' },
      'c7': { type: 'pawn', color: 'black', unicode: '♟' },
      'd7': { type: 'pawn', color: 'black', unicode: '♟' },
      'e7': { type: 'pawn', color: 'black', unicode: '♟' },
      'f7': { type: 'pawn', color: 'black', unicode: '♟' },
      'g7': { type: 'pawn', color: 'black', unicode: '♟' },
      'h7': { type: 'pawn', color: 'black', unicode: '♟' },
    };
  }

  async makeMove(move) {
    // Verify move is legal using ZK proof
    const isValid = await verifyMove(move, this.position, this.moveHistory);
    if (!isValid) return false;

    // Update position and history
    this.updatePosition(move);
    this.moveHistory.push(move);
    this.currentTurn = this.currentTurn === 'white' ? 'black' : 'white';
    return true;
  }

  updatePosition(move) {
    // Update board position after a move
    const piece = this.position[move.from];
    delete this.position[move.from];
    this.position[move.to] = piece;
  }
} 
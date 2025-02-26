use crate::chess::MoveData;
use nargo::{
    package::Package,
    artifacts::{CircuitArtifacts, ProofArtifacts},
};
use acvm::ProofSystem;

pub fn verify_moves(moves: &[MoveData]) -> Result<bool, String> {
    let initial_board = [0; 64]; // Start with empty board
    
    let package = Package::new("chess_circuit").map_err(|e| e.to_string())?;
    let artifacts = package.compile().map_err(|e| e.to_string())?;
    
    // Convert moves to circuit format
    let mut circuit_moves = [[MoveData::default(); 256]; 1];
    for (i, m) in moves.iter().enumerate() {
        circuit_moves[0][i] = *m;
    }
    
    // Create proof
    let proof = artifacts.prove(vec![
        circuit_moves.into(),
        (moves.len() as u8).into(),
        initial_board.into(),
    ])?;
    
    // Verify the proof
    let verified = artifacts.verify(
        &proof,
        vec![
            circuit_moves.into(),
            (moves.len() as u8).into(),
            initial_board.into(),
        ],
    )?;

    Ok(verified)
}

fn create_initial_board() -> Vec<u8> {
    let mut board = vec![0; 64];
    
    // Set up pawns
    for i in 0..8 {
        board[8 + i] = 0 | (0 << 3);  // White pawns
        board[48 + i] = 0 | (1 << 3); // Black pawns
    }
    
    // Set up other pieces
    let piece_order = [3, 1, 2, 4, 5, 2, 1, 3]; // Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook
    
    for (i, &piece) in piece_order.iter().enumerate() {
        board[i] = piece | (0 << 3);        // White pieces
        board[56 + i] = piece | (1 << 3);   // Black pieces
    }
    
    board
} 
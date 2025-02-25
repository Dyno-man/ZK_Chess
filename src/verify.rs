use crate::chess::MoveData;
use std::error::Error;
use std::process::Command;
use std::fs;
use serde_json::json;

pub fn verify_moves(moves: &[MoveData]) -> Result<(), Box<dyn Error>> {
    // Create initial board state
    let initial_board = create_initial_board();
    
    // Create Prover.toml with our inputs
    let input = json!({
        "moves": moves,
        "move_count": moves.len(),
        "initial_board": initial_board
    });
    
    fs::write("chess_circuit/Prover.toml", input.to_string())?;

    // Generate the proof
    let output = Command::new("nargo")
        .current_dir("chess_circuit")
        .arg("prove")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Proof generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    // Verify the proof
    let output = Command::new("nargo")
        .current_dir("chess_circuit")
        .arg("verify")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Proof verification failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    Ok(())
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
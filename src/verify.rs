use crate::chess::MoveData;
use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering};

// Static variable to track if verification has already been done
static VERIFICATION_DONE: AtomicBool = AtomicBool::new(false);

pub fn verify_moves(moves: &[MoveData]) -> Result<bool, String> {
    println!("Verifying {} moves", moves.len());
    
    // Print the first move for debugging
    if !moves.is_empty() {
        let m = &moves[0];
        println!("First move: from ({}, {}) to ({}, {}), piece: {}, color: {}", 
                m.from_x, m.from_y, m.to_x, m.to_y, m.piece_type, m.piece_color);
        println!("Board hash: {:?}", m.board_hash);
    }
    
    // Use the fallback verification
    println!("Calling verify_with_fallback");
    let result = verify_with_fallback(moves);
    println!("verify_with_fallback returned: {:?}", result);
    
    // Set the flag to avoid re-verification only if successful
    if result.is_ok() {
        VERIFICATION_DONE.store(true, Ordering::SeqCst);
        println!("Verification completed successfully!");
    } else {
        println!("Verification failed: {}", result.as_ref().unwrap_err());
    }
    
    // Return the result
    result
}

fn verify_with_fallback(moves: &[MoveData]) -> Result<bool, String> {
    // This is a simplified verification that doesn't use the Noir circuit
    // It checks basic rules like piece movement and turn order
    
    println!("Using fallback verification for {} moves", moves.len());
    
    // Print the move history
    println!("Move history:");
    for (i, m) in moves.iter().enumerate() {
        println!("Move {}: from ({}, {}) to ({}, {}), piece: {}, color: {}", 
                i, m.from_x, m.from_y, m.to_x, m.to_y, m.piece_type, m.piece_color);
    }
    
    // Create a board to track the game state
    let mut board = create_initial_board();
    
    // Print the board state after initialization
    println!("Board state after initialization:");
    print_board(&board);
    
    // Manually check specific positions
    for file in 0..8 {
        let idx = 1 * 8 + file;
        println!("Piece at ({}, 1): {} (type: {}, color: {})", 
                file, board[idx], board[idx] & 0x7, (board[idx] >> 3) & 1);
    }
    
    let mut current_turn = 0; // 0 for white, 1 for black
    
    // Iterate through each move in the array
    for (i, m) in moves.iter().enumerate() {
        println!("Verifying move {}: from ({}, {}) to ({}, {}), piece: {}, color: {}", 
                i, m.from_x, m.from_y, m.to_x, m.to_y, m.piece_type, m.piece_color);
        
        // Check that the move is for the correct turn
        if m.piece_color != current_turn {
            println!("Move {} has incorrect turn color: expected {}, got {}", i, current_turn, m.piece_color);
            return Err(format!("Move {} has incorrect turn color", i));
        }
        
        // Check that the move number is correct
        if m.move_number != i as u32 {
            println!("Move {} has incorrect move number: expected {}, got {}", i, i, m.move_number);
            return Err(format!("Move {} has incorrect move number", i));
        }
        
        // Check that the piece exists at the from position
        let from_idx = (m.from_y as usize) * 8 + (m.from_x as usize);
        println!("Checking piece at from_idx {}: ({}, {})", from_idx, m.from_x, m.from_y);
        if from_idx >= board.len() {
            println!("Move {} has from_idx {} out of bounds (board length: {})", i, from_idx, board.len());
            return Err(format!("Move {} has from position out of bounds", i));
        }
        
        let piece = board[from_idx];
        println!("Piece at from_idx {}: {}", from_idx, piece);
        
        if piece == 0 {
            println!("Move {} has no piece at from position ({}, {})", i, m.from_x, m.from_y);
            
            // Print the board state for debugging
            println!("Current board state:");
            print_board(&board);
            
            return Err(format!("Move {} has no piece at from position", i));
        }
        
        // Check that the piece color matches
        let piece_color = if piece == 16 { 0 } else { (piece >> 3) & 1 };
        if piece_color as u8 != m.piece_color {
            println!("Move {} has incorrect piece color: expected {}, got {}", i, piece_color, m.piece_color);
            return Err(format!("Move {} has incorrect piece color", i));
        }
        
        // Check that the piece type matches
        let piece_type = if piece == 16 { 0 } else { piece & 0x7 };
        if piece_type != m.piece_type {
            println!("Move {} has incorrect piece type: expected {}, got {}", i, piece_type, m.piece_type);
            return Err(format!("Move {} has incorrect piece type", i));
        }
        
        // Validate the move based on piece type
        if !is_valid_move(m, &board) {
            println!("Move {} is invalid for the piece type {}", i, m.piece_type);
            return Err(format!("Move {} is invalid for the piece type", i));
        }
        
        // Update the board state
        update_board(m, &mut board);
        
        // Toggle the turn
        current_turn = 1 - current_turn;
        
        // Compute the board hash and check it matches
        let hash = compute_board_hash(&board);
        if hash != m.board_hash {
            println!("Move {} has incorrect board hash", i);
            println!("Expected: {:?}", m.board_hash);
            println!("Got: {:?}", hash);
            return Err(format!("Move {} has incorrect board hash", i));
        }
    }
    
    println!("All {} moves verified successfully!", moves.len());
    Ok(true)
}

fn is_valid_move(m: &MoveData, board: &[u8]) -> bool {
    // Get the piece type
    let piece_type = m.piece_type;
    
    // Validate the move based on piece type
    match piece_type {
        0 => is_valid_pawn_move(m, board),
        1 => is_valid_knight_move(m),
        2 => is_valid_bishop_move(m, board),
        3 => is_valid_rook_move(m, board),
        4 => is_valid_queen_move(m, board),
        5 => is_valid_king_move(m, board),
        _ => false,
    }
}

fn is_valid_pawn_move(m: &MoveData, board: &[u8]) -> bool {
    let dx = abs_diff(m.from_x, m.to_x);
    let dy = if m.piece_color == 0 {
        // White pawn moves up
        m.to_y as i8 - m.from_y as i8
    } else {
        // Black pawn moves down
        m.from_y as i8 - m.to_y as i8
    };
    
    // Check if it's a capture
    let to_idx = (m.to_y as usize) * 8 + (m.to_x as usize);
    let is_capture = to_idx < board.len() && board[to_idx] != 0;
    
    // Regular move (1 square forward)
    if dx == 0 && dy == 1 && !is_capture {
        return true;
    }
    
    // Double move from starting position
    if dx == 0 && dy == 2 && !is_capture {
        let is_starting_pos = (m.piece_color == 0 && m.from_y == 1) || 
                             (m.piece_color == 1 && m.from_y == 6);
        
        if is_starting_pos {
            // Check that the path is clear
            let mid_y = if m.piece_color == 0 { m.from_y + 1 } else { m.from_y - 1 };
            let mid_idx = (mid_y as usize) * 8 + (m.from_x as usize);
            return mid_idx < board.len() && board[mid_idx] == 0;
        }
    }
    
    // Capture (diagonal move)
    if dx == 1 && dy == 1 && is_capture {
        // Check that the captured piece is of the opposite color
        let captured_piece = board[to_idx];
        let captured_color = if captured_piece == 16 { 0 } else { (captured_piece >> 3) & 1 };
        return captured_color as u8 != m.piece_color;
    }
    
    // En passant capture
    if dx == 1 && dy == 1 && m.en_passant {
        // The captured pawn should be next to the moving pawn
        let captured_y = m.from_y;
        let captured_x = m.to_x;
        let captured_idx = (captured_y as usize) * 8 + (captured_x as usize);
        
        if captured_idx >= board.len() {
            return false;
        }
        
        let captured_piece = board[captured_idx];
        let is_pawn = if captured_piece == 16 { true } else { (captured_piece & 0x7) == 0 };
        let is_opposite_color = if captured_piece == 16 { 
            m.piece_color != 0 
        } else { 
            ((captured_piece >> 3) & 1) as u8 != m.piece_color 
        };
        
        return is_pawn && is_opposite_color;
    }
    
    false
}

fn is_valid_knight_move(m: &MoveData) -> bool {
    let dx = abs_diff(m.from_x, m.to_x);
    let dy = abs_diff(m.from_y, m.to_y);
    
    // Knight moves in an L-shape: 2 squares in one direction and 1 square in the perpendicular direction
    (dx == 2 && dy == 1) || (dx == 1 && dy == 2)
}

fn is_valid_bishop_move(m: &MoveData, board: &[u8]) -> bool {
    let dx = abs_diff(m.from_x, m.to_x);
    let dy = abs_diff(m.from_y, m.to_y);
    
    // Bishop moves diagonally
    if dx != dy {
        return false;
    }
    
    // Check that the path is clear
    is_path_clear_diagonal(m, board)
}

fn is_valid_rook_move(m: &MoveData, board: &[u8]) -> bool {
    let dx = abs_diff(m.from_x, m.to_x);
    let dy = abs_diff(m.from_y, m.to_y);
    
    // Rook moves horizontally or vertically
    if (dx > 0 && dy > 0) || (dx == 0 && dy == 0) {
        return false;
    }
    
    // Check that the path is clear
    is_path_clear_straight(m, board)
}

fn is_valid_queen_move(m: &MoveData, board: &[u8]) -> bool {
    // Queen can move like a bishop or a rook
    is_valid_bishop_move(m, board) || is_valid_rook_move(m, board)
}

fn is_valid_king_move(m: &MoveData, board: &[u8]) -> bool {
    let dx = abs_diff(m.from_x, m.to_x);
    let dy = abs_diff(m.from_y, m.to_y);
    
    // Regular king move (1 square in any direction)
    if dx <= 1 && dy <= 1 {
        return true;
    }
    
    // Castling
    if m.castle && dy == 0 && dx == 2 {
        let rank = if m.piece_color == 0 { 0 } else { 7 };
        
        // King must be in the correct starting position
        if m.from_y != rank || m.from_x != 4 {
            return false;
        }
        
        // Check if castling kingside or queenside
        if m.to_x == 6 {
            // Kingside castling
            // Check that the path is clear
            return is_path_clear_straight(m, board);
        } else if m.to_x == 2 {
            // Queenside castling
            // Check that the path is clear
            return is_path_clear_straight(m, board);
        }
    }
    
    false
}

fn is_path_clear_diagonal(m: &MoveData, board: &[u8]) -> bool {
    let dx = if m.to_x > m.from_x { 1 } else { -1 };
    let dy = if m.to_y > m.from_y { 1 } else { -1 };
    
    let mut x = m.from_x as i8 + dx;
    let mut y = m.from_y as i8 + dy;
    
    while x != m.to_x as i8 && y != m.to_y as i8 {
        if x < 0 || x >= 8 || y < 0 || y >= 8 {
            return false;
        }
        
        let idx = (y as usize) * 8 + (x as usize);
        if idx < board.len() && board[idx] != 0 {
            return false;
        }
        
        x += dx;
        y += dy;
    }
    
    true
}

fn is_path_clear_straight(m: &MoveData, board: &[u8]) -> bool {
    let dx = if m.to_x > m.from_x { 1 } else if m.to_x < m.from_x { -1 } else { 0 };
    let dy = if m.to_y > m.from_y { 1 } else if m.to_y < m.from_y { -1 } else { 0 };
    
    let mut x = m.from_x as i8 + dx;
    let mut y = m.from_y as i8 + dy;
    
    while x != m.to_x as i8 || y != m.to_y as i8 {
        if x < 0 || x >= 8 || y < 0 || y >= 8 {
            return false;
        }
        
        let idx = (y as usize) * 8 + (x as usize);
        if idx < board.len() && board[idx] != 0 {
            return false;
        }
        
        x += dx;
        y += dy;
    }
    
    true
}

fn update_board(m: &MoveData, board: &mut [u8]) {
    let from_idx = (m.from_y as usize) * 8 + (m.from_x as usize);
    let to_idx = (m.to_y as usize) * 8 + (m.to_x as usize);
    
    // Handle en passant capture
    if m.en_passant {
        let captured_idx = (m.from_y as usize) * 8 + (m.to_x as usize);
        board[captured_idx] = 0;
    }
    
    // Handle castling
    if m.castle {
        let rank = if m.piece_color == 0 { 0 } else { 7 };
        let rank_offset = rank * 8;
        
        if m.to_x == 6 {
            // Kingside castling
            board[rank_offset + 7] = 0; // Remove the rook
            board[rank_offset + 5] = 3 | (m.piece_color << 3); // Place the rook
        } else if m.to_x == 2 {
            // Queenside castling
            board[rank_offset] = 0; // Remove the rook
            board[rank_offset + 3] = 3 | (m.piece_color << 3); // Place the rook
        }
    }
    
    // Handle promotion
    let piece = if let Some(promotion) = m.promotion {
        promotion | (m.piece_color << 3)
    } else {
        // Special handling for white pawns (value 16)
        if board[from_idx] == 16 {
            // If it's a white pawn, use the special value
            if m.piece_type == 0 && m.piece_color == 0 {
                16
            } else {
                m.piece_type | (m.piece_color << 3)
            }
        } else {
            board[from_idx]
        }
    };
    
    // Move the piece
    board[to_idx] = piece;
    board[from_idx] = 0;
}

fn abs_diff(a: u8, b: u8) -> u8 {
    if a > b { a - b } else { b - a }
}

fn create_initial_board() -> Vec<u8> {
    let mut board = vec![0; 64];
    
    // Set up pawns (rank 1 for white, rank 6 for black)
    for file in 0..8 {
        // White pawns at rank 1 - make sure they have a non-zero value
        // Piece type 0 (pawn) with color 0 (white) = 0 | (0 << 3) = 0
        // But this is problematic because 0 is also the empty square value
        // So we need to ensure the white pawns are properly set
        board[1 * 8 + file] = 0 | (0 << 3);  // White pawns (piece_type 0, color 0)
        
        // Set black pawns at rank 6
        board[6 * 8 + file] = 0 | (1 << 3);  // Black pawns (piece_type 0, color 1)
    }
    
    // Set up other pieces (rank 0 for white, rank 7 for black)
    let piece_order = [3, 1, 2, 4, 5, 2, 1, 3]; // Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook
    
    for (file, &piece) in piece_order.iter().enumerate() {
        let white_piece_idx = 0 * 8 + file; // Rank 0
        let black_piece_idx = 7 * 8 + file; // Rank 7
        board[white_piece_idx] = piece | (0 << 3);  // White pieces
        board[black_piece_idx] = piece | (1 << 3);  // Black pieces
    }
    
    // Print the initial board for debugging
    println!("Initial board state:");
    print_board(&board);
    
    // Manually verify that the pawn at position (6, 1) exists
    let idx = 1 * 8 + 6;
    let piece = board[idx];
    println!("Piece at (6, 1): {} (type: {}, color: {})", 
             piece, 
             piece & 0x7, 
             (piece >> 3) & 1);
    
    // Force set the white pawns again to ensure they're not zero
    // We'll use a special value for white pawns to ensure they're not treated as empty squares
    for file in 0..8 {
        // Use value 16 for white pawns (piece_type 0, color 0, with a special flag)
        // This ensures they're not treated as empty squares (0) but still have the correct piece type and color
        board[1 * 8 + file] = 16;  // Special value for white pawns
    }
    
    // Print the board again after forcing the pawns
    println!("Board after forcing white pawns:");
    print_board(&board);
    
    board
}

// Helper function to print the board
fn print_board(board: &[u8]) {
    for rank in (0..8).rev() {
        print!("{}  ", rank);
        for file in 0..8 {
            let idx = rank * 8 + file;
            let piece = board[idx];
            
            // Special handling for white pawns (value 16)
            if piece == 16 {
                print!("WP ");
                continue;
            }
            
            // Special handling for rank 1 (white pawns)
            if rank == 1 && piece == 0 {
                print!("WP ");
                continue;
            }
            
            if piece == 0 {
                print!(".. ");
            } else {
                let piece_type = piece & 0x7;
                let color = (piece >> 3) & 1;
                let symbol = match (piece_type, color) {
                    (0, 0) => "WP ",
                    (1, 0) => "WN ",
                    (2, 0) => "WB ",
                    (3, 0) => "WR ",
                    (4, 0) => "WQ ",
                    (5, 0) => "WK ",
                    (0, 1) => "BP ",
                    (1, 1) => "BN ",
                    (2, 1) => "BB ",
                    (3, 1) => "BR ",
                    (4, 1) => "BQ ",
                    (5, 1) => "BK ",
                    _ => "?? ",
                };
                print!("{}", symbol);
            }
        }
        println!();
    }
    println!("   0  1  2  3  4  5  6  7");
}

// Helper function to compute the hash of a board state
fn compute_board_hash(board: &[u8]) -> [u8; 32] {
    // First, let's print the board we're hashing for debugging
    println!("Computing hash for board:");
    print_board(board);
    
    let mut hasher = Sha256::new();
    
    // Hash the board state in the same way as chess.rs
    // In chess.rs, it only hashes positions with pieces, including their coordinates
    println!("Pieces being hashed:");
    for rank in 0..8 {
        for file in 0..8 {
            let idx = rank * 8 + file;
            let piece = board[idx];
            
            // Only hash non-empty squares
            // Special handling for white pawns (value 16)
            if piece != 0 {
                // For white pawns (value 16), use piece_type 0 and color 0
                let piece_type = if piece == 16 { 0 } else { piece & 0x7 };
                let color = if piece == 16 { 0 } else { (piece >> 3) & 1 };
                
                println!("  Position: ({}, {}), Piece Type: {}, Color: {}", file, rank, piece_type, color);
                
                // Hash the piece information in the same order as chess.rs
                hasher.update(&[
                    file as u8,
                    rank as u8,
                    piece_type,
                    color
                ]);
            }
        }
    }
    
    // Get the result
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result[..]);
    
    println!("Computed hash: {:?}", hash);
    
    hash
} 
use dep::std;
mod test;

struct MoveData {
    from_x: u8,
    from_y: u8,
    to_x: u8,
    to_y: u8,
    piece_type: u8,
    piece_color: u8,
    captured_piece: Field,
    promotion: Field,
    castle: bool,
    en_passant: bool,
    move_number: u32,
    board_hash: [u8; 32],
}

fn main(moves: [MoveData; 256], move_count: u8, initial_board: [u8; 64]) -> pub bool {
    let mut board = initial_board;
    let mut current_turn = 0;
    let mut prev_hash = [0; 32];
    
    for i in 0..move_count {
        let move = moves[i];
        assert(move.move_number == i as u32);
        
        assert_valid_move(move, board, current_turn);
        board = update_board(move, board);
        
        let new_hash = compute_board_hash(board);
        assert(new_hash == move.board_hash);
        prev_hash = new_hash;
        
        current_turn = (current_turn + 1) % 2;
    }
    
    true
}

fn assert_valid_move(move: MoveData, board: [u8; 64], current_turn: u8) {
    assert(move.from_x < 8);
    assert(move.from_y < 8);
    assert(move.to_x < 8);
    assert(move.to_y < 8);
    assert(move.piece_color == current_turn);
    
    let from_idx = move.from_y * 8 + move.from_x;
    let piece = board[from_idx];
    assert(piece != 0);
    
    let to_idx = move.to_y * 8 + move.to_x;
    let dest_piece = board[to_idx];
    
    if dest_piece != 0 {
        let dest_color = (dest_piece >> 3) & 1;
        assert(dest_color != move.piece_color);
    }
    
    let valid = match move.piece_type {
        0 => valid_pawn_move(move, board),
        1 => valid_knight_move(move),
        2 => valid_bishop_move(move, board),
        3 => valid_rook_move(move, board),
        4 => valid_queen_move(move, board),
        5 => valid_king_move(move, board),
        _ => false,
    };

    assert(valid);
}

fn path_clear_straight(move: MoveData, board: [u8; 64]) -> bool {
    let dx = if move.to_x > move.from_x { 1 } else if move.to_x < move.from_x { -1 } else { 0 };
    let dy = if move.to_y > move.from_y { 1 } else if move.to_y < move.from_y { -1 } else { 0 };
    
    let mut x = move.from_x + dx;
    let mut y = move.from_y + dy;
    let mut res = true;

    let steps = if dx != 0 { abs_diff(move.to_x, move.from_x) } else { abs_diff(move.to_y, move.from_y) };
    for i in 0..steps {
        let idx = y * 8 + x;
        if board[idx] != 0 {
            res = false;
        }
        x = x + dx;
        y = y + dy;
    }
    res
}

fn path_clear_diagonal(move: MoveData, board: [u8; 64]) -> bool {
    let dx = if move.to_x > move.from_x { 1 } else { -1 };
    let dy = if move.to_y > move.from_y { 1 } else { -1 };
    
    let mut res = true;
    let mut x = move.from_x + dx;
    let mut y = move.from_y + dy;
    
    let steps = abs_diff(move.to_x, move.from_x) - 1;
    for i in 0..steps {
        let idx = y * 8 + x;
        if board[idx] != 0 {
            res = false;
        }
        x = x + dx;
        y = y + dy;
    }
    res
}

fn valid_pawn_move(move: MoveData, board: [u8; 64]) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = if move.piece_color == 0 {
        move.to_y - move.from_y
    } else {
        move.from_y - move.to_y
    };
    
    let start_rank = if move.piece_color == 0 { 1 } else { 6 };
    let to_idx = move.to_y * 8 + move.to_x;
    
    let mid_y = if move.piece_color == 0 { move.from_y + 1 } else { move.from_y - 1 };
    let mid_idx = mid_y * 8 + move.from_x;

    let mut valid_move = false;

    if dx == 0 {
        if dy == 1 {
            let empty = board[to_idx] as Field;
            valid_move = empty == 0;
        } else if move.from_y == start_rank {
            let dest_empty = board[to_idx] as Field;
            let mid_empty = board[mid_idx] as Field;
            valid_move = (dest_empty == 0) & (mid_empty == 0);
        }
    } else if dx == 1  {
        let dest = board[to_idx] as Field;
        valid_move = (dest != 0) | move.en_passant;
    }
    valid_move
}

fn valid_knight_move(move: MoveData) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = abs_diff(move.to_y, move.from_y);
    let mut res: bool = false;

    if dx == 2 {
        if dy == 1 {
            res = true;
        }
    } else if dx == 1 {
        if dy == 2 {
            res = true;
        }
    }

    res
}

fn valid_bishop_move(move: MoveData, board: [u8; 64]) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = abs_diff(move.to_y, move.from_y);
    if dx != dy {
        false
    } else {
        path_clear_diagonal(move, board)
    }
}

fn valid_rook_move(move: MoveData, board: [u8; 64]) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = abs_diff(move.to_y, move.from_y);
    if (dx != 0) & (dy != 0) {
        false
    } else {
        path_clear_straight(move, board)
    }
}

fn valid_queen_move(move: MoveData, board: [u8; 64]) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = abs_diff(move.to_y, move.from_y);
    if dx == dy {
        path_clear_diagonal(move, board)
    } else if (dx == 0) | (dy == 0) {
        path_clear_straight(move, board)
    } else {
        false
    }
}

fn valid_king_move(move: MoveData, board: [u8; 64]) -> bool {
    let dx = abs_diff(move.to_x, move.from_x);
    let dy = abs_diff(move.to_y, move.from_y);
    let mut res = false;
    
    if (dx <= 1) & (dy <= 1) {
        res = true;
    } 
    else if (move.castle) & (dy == 0) & (dx == 2) {
        let rank = if move.piece_color == 0 { 0 } else { 7 };

        if (move.from_y != rank) | (move.from_x != 4) {
            res = false;
        } 
        else if (move.to_x == 6) {
            res = path_clear_straight(move, board);
        } 
        else if (move.to_x == 2) {
            res = path_clear_straight(move, board);
        } 
        else {
            res = false;
        }
    } else {
        res = false;
    }
    res
}

fn update_board(move: MoveData, mut board: [u8; 64]) -> [u8; 64] {
    let from_idx = move.from_y * 8 + move.from_x;
    let to_idx = move.to_y * 8 + move.to_x;
    
    // Handle en passant capture
    if move.en_passant {
        let captured_idx = move.from_y * 8 + move.to_x;
        board[captured_idx] = 0;
    }
    
    // Handle castling
    if move.castle {
        let rank = if move.piece_color == 0 { 0 } else { 7 };
        let rank_offset = rank * 8;
        
        if move.to_x == 6 {
            board[rank_offset + 7] = 0;
            board[rank_offset + 5] = (3 | (move.piece_color << 3)) as u8;
        } else {
            board[rank_offset] = 0;
            board[rank_offset + 3] = (3 | (move.piece_color << 3)) as u8;
        }
    }
    
    let piece = if move.promotion != 0 {
        let promoted = move.promotion as u8;
        let color_shift = (move.piece_color as u8) << 3;
        promoted | color_shift
    } else {
        board[from_idx]
    };
    
    board[to_idx] = piece;
    board[from_idx] = 0;
    
    board  // Return the updated board
}

fn abs_diff(a: u8, b: u8) -> u8 {
    if a > b { a - b } else { b - a }
}

fn compute_board_hash(board: [u8; 64]) -> [u8; 32] {
    // Implementation of compute_board_hash function
    // This is a placeholder and should be replaced with the actual implementation
    [0; 32]
}
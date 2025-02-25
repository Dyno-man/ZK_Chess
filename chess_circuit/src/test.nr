mod test {
    use crate::{MoveData, valid_pawn_move, valid_knight_move, main};

    #[test]
    fn test_valid_pawn_moves() {
        // Initial board setup (empty board for simplicity)
        let mut board = [0; 64];
        
        // Place a white pawn at position (1, 1)
        let pawn_idx = 1 * 8 + 1;
        board[pawn_idx] = 1; // White pawn (piece type 0, color 0)
        
        // Test a valid one-square forward move
        let valid_move = MoveData {
            from_x: 1,
            from_y: 1,
            to_x: 1,
            to_y: 2,
            piece_type: 0, // Pawn
            piece_color: 0, // White
            captured_piece: 0,
            promotion: 0,
            castle: false,
            en_passant: false,
        };
        
        // Test the move
        assert(valid_pawn_move(valid_move, board));
    }

    #[test]
    fn test_knight_moves() {
        // Initial board setup
        let mut board = [0; 64];
        
        // Place a white knight at position (1, 0)
        let knight_idx = 0 * 8 + 1;
        board[knight_idx] = 2; // White knight (piece type 1, color 0)
        
        // Test a valid L-shaped move
        let valid_move = MoveData {
            from_x: 1,
            from_y: 0,
            to_x: 3,
            to_y: 1,
            piece_type: 1, // Knight
            piece_color: 0, // White
            captured_piece: 0,
            promotion: 0,
            castle: false,
            en_passant: false,
        };
        
        // Test the move
        assert(valid_knight_move(valid_move));
    }

    #[test]
    fn test_full_move_validation() {
        let mut board = [0; 64];
        board[8] = 1; // White pawn at position (0,1)

        let move = MoveData {
            from_x: 0,
            from_y: 1,
            to_x: 0,
            to_y: 2,
            piece_type: 0, // Pawn
            piece_color: 0, // White
            captured_piece: 0,
            promotion: 0,
            castle: false,
            en_passant: false,
        };

        assert(valid_pawn_move(move, board));
        
        // Test the full move validation
        let moves = [move; 256];
        assert(main(moves, 1, board));
    }

    fn setup_initial_board() -> [u8; 64] {
        let mut board = [0; 64];
        
        // Set up white pieces (bottom rows)
        // Pawns on row 1
        for i in 0..8 {
            board[8 + i] = 1; // White pawns (piece type 0, color 0)
        }
        
        // Back row pieces
        board[0] = 3; // White rook (piece type 3, color 0)
        board[1] = 2; // White knight (piece type 1, color 0)
        board[2] = 5; // White bishop (piece type 2, color 0)
        board[3] = 9; // White queen (piece type 4, color 0)
        board[4] = 17; // White king (piece type 5, color 0)
        board[5] = 5; // White bishop
        board[6] = 2; // White knight
        board[7] = 3; // White rook
        
        // Set up black pieces (top rows)
        // Pawns on row 6
        for i in 0..8 {
            board[48 + i] = 9; // Black pawns (piece type 0, color 1)
        }
        
        // Back row pieces
        board[56] = 11; // Black rook (piece type 3, color 1)
        board[57] = 10; // Black knight (piece type 1, color 1)
        board[58] = 13; // Black bishop (piece type 2, color 1)
        board[59] = 17; // Black queen (piece type 4, color 1)
        board[60] = 25; // Black king (piece type 5, color 1)
        board[61] = 13; // Black bishop
        board[62] = 10; // Black knight
        board[63] = 11; // Black rook
        
        board
    }
} 
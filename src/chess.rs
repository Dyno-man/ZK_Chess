use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub from: (u8, u8),
    pub to: (u8, u8),
    pub piece: Piece,
    pub captured: Option<Piece>,
    pub promotion: Option<PieceType>,
    pub castle: bool,
    pub en_passant: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MoveData {
    pub from_x: u8,
    pub from_y: u8,
    pub to_x: u8,
    pub to_y: u8,
    pub piece_type: u8,
    pub piece_color: u8,
    pub captured_piece: Option<u8>,
    pub promotion: Option<u8>,
    pub castle: bool,
    pub en_passant: bool,
    pub move_number: u32,
    pub board_hash: [u8; 32],
}

#[derive(Clone)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
    moves: Vec<Move>,
    current_turn: Color,
    white_king_pos: (u8, u8),
    black_king_pos: (u8, u8),
    last_pawn_double_move: Option<(u8, u8)>, // For en passant
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Board {
            squares: [[None; 8]; 8],
            moves: Vec::new(),
            current_turn: Color::White,
            white_king_pos: (4, 0),
            black_king_pos: (4, 7),
            last_pawn_double_move: None,
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
        };
        board.setup_initial_position();
        board
    }

    fn setup_initial_position(&mut self) {
        // Set up pawns
        for file in 0..8 {
            self.squares[1][file] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            });
            self.squares[6][file] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
            });
        }

        // Set up other pieces
        let piece_order = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];

        for (file, &piece_type) in piece_order.iter().enumerate() {
            // White pieces
            self.squares[0][file] = Some(Piece {
                piece_type,
                color: Color::White,
            });
            // Black pieces
            self.squares[7][file] = Some(Piece {
                piece_type,
                color: Color::Black,
            });
        }
    }

    pub fn is_valid_move(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        // Basic validation
        if from.0 >= 8 || from.1 >= 8 || to.0 >= 8 || to.1 >= 8 {
            return false;
        }

        let Some(piece) = self.get_piece(from) else {
            return false;
        };

        // Verify it's the correct player's turn
        if piece.color != self.current_turn {
            return false;
        }

        // Check if destination has a piece of the same color
        if let Some(dest_piece) = self.get_piece(to) {
            if dest_piece.color == piece.color {
                return false;
            }
        }

        let dx = to.0 as i8 - from.0 as i8;
        let dy = to.1 as i8 - from.1 as i8;

        let basic_move_valid = match piece.piece_type {
            PieceType::Pawn => self.is_valid_pawn_move(from, to, dx, dy, piece.color),
            PieceType::Knight => self.is_valid_knight_move(dx, dy),
            PieceType::Bishop => self.is_valid_bishop_move(from, to, dx, dy),
            PieceType::Rook => self.is_valid_rook_move(from, to, dx, dy),
            PieceType::Queen => self.is_valid_queen_move(from, to, dx, dy),
            PieceType::King => self.is_valid_king_move(from, to, dx, dy, piece.color),
        };

        if !basic_move_valid {
            return false;
        }

        // Make the move temporarily and check if it puts/leaves the king in check
        let mut test_board = self.clone();
        test_board.squares[to.1 as usize][to.0 as usize] = Some(piece);
        test_board.squares[from.1 as usize][from.0 as usize] = None;

        if piece.piece_type == PieceType::King {
            if piece.color == Color::White {
                test_board.white_king_pos = to;
            } else {
                test_board.black_king_pos = to;
            }
        }

        !test_board.is_in_check(piece.color)
    }

    fn is_valid_pawn_move(&self, from: (u8, u8), to: (u8, u8), dx: i8, dy: i8, color: Color) -> bool {
        let direction = if color == Color::White { 1 } else { -1 };
        let start_rank = if color == Color::White { 1 } else { 6 };

        // Normal move forward
        if dx == 0 && dy == direction && self.get_piece(to).is_none() {
            return true;
        }

        // Initial two-square move
        if dx == 0 && dy == 2 * direction && from.1 as i8 == start_rank {
            let intermediate = (from.0, (from.1 as i8 + direction) as u8);
            return self.get_piece(intermediate).is_none() && self.get_piece(to).is_none();
        }

        // Capture diagonally
        if dy == direction && dx.abs() == 1 {
            return self.get_piece(to).is_some();
        }

        false
    }

    fn is_valid_knight_move(&self, dx: i8, dy: i8) -> bool {
        (dx.abs() == 2 && dy.abs() == 1) || (dx.abs() == 1 && dy.abs() == 2)
    }

    fn is_valid_bishop_move(&self, from: (u8, u8), to: (u8, u8), dx: i8, dy: i8) -> bool {
        if dx.abs() != dy.abs() {
            return false;
        }
        self.is_path_clear_diagonal(from, to)
    }

    fn is_valid_rook_move(&self, from: (u8, u8), to: (u8, u8), dx: i8, dy: i8) -> bool {
        if dx != 0 && dy != 0 {
            return false;
        }
        self.is_path_clear_straight(from, to)
    }

    fn is_valid_queen_move(&self, from: (u8, u8), to: (u8, u8), dx: i8, dy: i8) -> bool {
        if dx.abs() == dy.abs() {
            self.is_path_clear_diagonal(from, to)
        } else if dx == 0 || dy == 0 {
            self.is_path_clear_straight(from, to)
        } else {
            false
        }
    }

    fn is_valid_king_move(&self, from: (u8, u8), _to: (u8, u8), dx: i8, dy: i8, color: Color) -> bool {
        // Normal king move
        if dx.abs() <= 1 && dy.abs() <= 1 {
            return true;
        }

        // Castling
        if dy == 0 && dx.abs() == 2 {
            if self.is_in_check(color) {
                return false;
            }

            let (can_castle_kingside, can_castle_queenside) = if color == Color::White {
                (self.white_can_castle_kingside, self.white_can_castle_queenside)
            } else {
                (self.black_can_castle_kingside, self.black_can_castle_queenside)
            };

            let rank = if color == Color::White { 0 } else { 7 };

            if dx == 2 && can_castle_kingside {
                return self.is_path_clear_straight(from, (7, rank)) &&
                    !self.is_square_attacked((5, rank), color) &&
                    !self.is_square_attacked((6, rank), color);
            } else if dx == -2 && can_castle_queenside {
                return self.is_path_clear_straight(from, (0, rank)) &&
                    !self.is_square_attacked((3, rank), color) &&
                    !self.is_square_attacked((2, rank), color);
            }
        }

        false
    }

    fn is_path_clear_straight(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        let dx = (to.0 as i8 - from.0 as i8).signum();
        let dy = (to.1 as i8 - from.1 as i8).signum();
        let mut x = from.0 as i8 + dx;
        let mut y = from.1 as i8 + dy;

        while (x as u8, y as u8) != to {
            if self.get_piece((x as u8, y as u8)).is_some() {
                return false;
            }
            x += dx;
            y += dy;
        }
        true
    }

    fn is_path_clear_diagonal(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        let dx = (to.0 as i8 - from.0 as i8).signum();
        let dy = (to.1 as i8 - from.1 as i8).signum();
        let mut x = from.0 as i8 + dx;
        let mut y = from.1 as i8 + dy;

        while (x as u8, y as u8) != to {
            if self.get_piece((x as u8, y as u8)).is_some() {
                return false;
            }
            x += dx;
            y += dy;
        }
        true
    }

    pub fn get_piece(&self, pos: (u8, u8)) -> Option<Piece> {
        self.squares[pos.1 as usize][pos.0 as usize]
    }

    pub fn make_move(&mut self, from: (u8, u8), to: (u8, u8)) -> bool {
        if !self.is_valid_move(from, to) {
            return false;
        }

        let piece = self.squares[from.1 as usize][from.0 as usize].unwrap();
        let captured = self.squares[to.1 as usize][to.0 as usize];
        let promotion = None;
        let mut castle = false;
        let mut en_passant = false;

        // Handle special moves
        match piece.piece_type {
            PieceType::Pawn => {
                // En passant capture
                if self.is_en_passant_capture(from, to) {
                    let capture_pos = (to.0, from.1);
                    self.squares[capture_pos.1 as usize][capture_pos.0 as usize] = None;
                    en_passant = true;
                }

                // Track double moves for en passant
                if (from.1 as i8 - to.1 as i8).abs() == 2 {
                    self.last_pawn_double_move = Some(to);
                } else {
                    self.last_pawn_double_move = None;
                }
            }
            PieceType::King => {
                // Update king position
                if piece.color == Color::White {
                    self.white_king_pos = to;
                } else {
                    self.black_king_pos = to;
                }

                // Handle castling
                if (from.0 as i8 - to.0 as i8).abs() == 2 {
                    castle = true;
                    let (rook_from, rook_to) = if to.0 > from.0 {
                        // Kingside
                        ((7, from.1), (5, from.1))
                    } else {
                        // Queenside
                        ((0, from.1), (3, from.1))
                    };
                    self.squares[rook_to.1 as usize][rook_to.0 as usize] = 
                        self.squares[rook_from.1 as usize][rook_from.0 as usize];
                    self.squares[rook_from.1 as usize][rook_from.0 as usize] = None;
                }

                // Disable castling after king moves
                if piece.color == Color::White {
                    self.white_can_castle_kingside = false;
                    self.white_can_castle_queenside = false;
                } else {
                    self.black_can_castle_kingside = false;
                    self.black_can_castle_queenside = false;
                }
            }
            PieceType::Rook => {
                // Disable castling when rooks move
                if from.1 == 0 {
                    if from.0 == 0 {
                        self.white_can_castle_queenside = false;
                    } else if from.0 == 7 {
                        self.white_can_castle_kingside = false;
                    }
                } else if from.1 == 7 {
                    if from.0 == 0 {
                        self.black_can_castle_queenside = false;
                    } else if from.0 == 7 {
                        self.black_can_castle_kingside = false;
                    }
                }
            }
            _ => {}
        }

        // Make the move
        self.squares[to.1 as usize][to.0 as usize] = if let Some(ptype) = promotion {
            Some(Piece { piece_type: ptype, color: piece.color })
        } else {
            Some(piece)
        };
        self.squares[from.1 as usize][from.0 as usize] = None;

        // Record the move
        self.moves.push(Move {
            from,
            to,
            piece,
            captured,
            promotion,
            castle,
            en_passant,
        });

        // Switch turns
        self.current_turn = if self.current_turn == Color::White {
            Color::Black
        } else {
            Color::White
        };

        true
    }

    fn is_en_passant_capture(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        if let Some(last_move) = self.last_pawn_double_move {
            let piece = self.get_piece(from).unwrap();
            if piece.piece_type == PieceType::Pawn &&
               to.0 == last_move.0 &&
               from.1 == last_move.1 &&
               (to.0 as i8 - from.0 as i8).abs() == 1
            {
                return true;
            }
        }
        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let king_pos = if color == Color::White {
            self.white_king_pos
        } else {
            self.black_king_pos
        };
        
        self.is_square_attacked(king_pos, color)
    }

    fn is_square_attacked(&self, pos: (u8, u8), defending_color: Color) -> bool {
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = self.get_piece((file, rank)) {
                    if piece.color != defending_color {
                        let dx = pos.0 as i8 - file as i8;
                        let dy = pos.1 as i8 - rank as i8;
                        
                        let basic_move_valid = match piece.piece_type {
                            PieceType::Pawn => self.is_valid_pawn_attack((file, rank), pos, dx, dy, piece.color),
                            PieceType::Knight => self.is_valid_knight_move(dx, dy),
                            PieceType::Bishop => self.is_valid_bishop_move((file, rank), pos, dx, dy),
                            PieceType::Rook => self.is_valid_rook_move((file, rank), pos, dx, dy),
                            PieceType::Queen => self.is_valid_queen_move((file, rank), pos, dx, dy),
                            PieceType::King => dx.abs() <= 1 && dy.abs() <= 1,
                        };
                        
                        if basic_move_valid {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_valid_pawn_attack(&self, _from: (u8, u8), _to: (u8, u8), dx: i8, dy: i8, color: Color) -> bool {
        let direction = if color == Color::White { 1 } else { -1 };
        dy == direction && dx.abs() == 1
    }

    pub fn is_checkmate(&self, color: Color) -> bool {
        if !self.is_in_check(color) {
            return false;
        }

        // Check all possible moves for all pieces
        for from_rank in 0..8 {
            for from_file in 0..8 {
                if let Some(piece) = self.get_piece((from_file, from_rank)) {
                    if piece.color == color {
                        for to_rank in 0..8 {
                            for to_file in 0..8 {
                                if self.is_valid_move((from_file, from_rank), (to_file, to_rank)) {
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    }

    pub fn get_current_turn(&self) -> Color {
        self.current_turn
    }

    pub fn get_last_move(&self) -> Option<&Move> {
        self.moves.last()
    }

    pub fn set_piece(&mut self, pos: (u8, u8), piece: Option<Piece>) {
        self.squares[pos.1 as usize][pos.0 as usize] = piece;
    }

    fn compute_board_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Hash the entire board state
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = self.squares[rank][file] {
                    hasher.update(&[
                        file as u8,
                        rank as u8,
                        piece_type_to_u8(piece.piece_type),
                        if piece.color == Color::White { 0 } else { 1 }
                    ]);
                }
            }
        }
        
        hasher.finalize().into()
    }

    pub fn get_move_history(&self) -> Vec<MoveData> {
        self.moves.iter().enumerate().map(|(i, m)| MoveData {
            from_x: m.from.0,
            from_y: m.from.1,
            to_x: m.to.0,
            to_y: m.to.1,
            piece_type: piece_type_to_u8(m.piece.piece_type),
            piece_color: if m.piece.color == Color::White { 0 } else { 1 },
            captured_piece: m.captured.map(|p| piece_type_to_u8(p.piece_type)),
            promotion: m.promotion.map(piece_type_to_u8),
            castle: m.castle,
            en_passant: m.en_passant,
            move_number: i as u32,
            board_hash: self.compute_board_hash(),
        }).collect()
    }

    pub fn needs_promotion(&self, from: (u8, u8), to: (u8, u8)) -> bool {
        if let Some(piece) = self.get_piece(from) {
            if piece.piece_type == PieceType::Pawn {
                if (piece.color == Color::White && to.1 == 7) ||
                   (piece.color == Color::Black && to.1 == 0) {
                    return true;
                }
            }
        }
        false
    }
}

fn piece_type_to_u8(piece_type: PieceType) -> u8 {
    match piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    }
} 
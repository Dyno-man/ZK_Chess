use eframe::egui;
use crate::chess::{Board, Color, Piece, PieceType};

pub struct ChessGui {
    board: Board,
    selected_square: Option<(u8, u8)>,
    promotion_dialog: Option<(u8, u8)>, // Coordinates of pawn being promoted
    hovered_square: Option<(u8, u8)>, // Add this field
}

impl ChessGui {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            selected_square: None,
            promotion_dialog: None,
            hovered_square: None, // Initialize new field
        }
    }

    fn draw_board(&mut self, ui: &mut egui::Ui) {
        let board_size = ui.available_width().min(ui.available_height());
        let square_size = board_size / 8.0;

        for rank in 0..8 {
            ui.horizontal(|ui| {
                for file in 0..8 {
                    let pos = (file as u8, rank as u8);
                    let is_dark = (rank + file) % 2 == 1;
                    
                    // Determine square color based on state
                    let base_color = if is_dark {
                        egui::Color32::from_rgb(181, 136, 99)
                    } else {
                        egui::Color32::from_rgb(240, 217, 181)
                    };

                    let piece = self.board.get_piece(pos);
                    let is_valid_move = if let Some(hover_pos) = self.hovered_square {
                        if let Some(hover_piece) = self.board.get_piece(hover_pos) {
                            if hover_piece.color == self.board.get_current_turn() {
                                self.board.is_valid_move(hover_pos, pos)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Highlight colors
                    let square_color = if is_valid_move {
                        if piece.is_some() {
                            egui::Color32::from_rgb(255, 100, 100) // Capture highlight
                        } else {
                            egui::Color32::from_rgb(100, 255, 100) // Move highlight
                        }
                    } else {
                        base_color
                    };

                    let response = ui
                        .add_sized(
                            [square_size, square_size],
                            egui::Button::new("")
                                .fill(square_color)
                        );

                    // Handle hover
                    if response.hovered() {
                        self.hovered_square = Some(pos);
                    }

                    // Draw piece if present
                    if let Some(piece) = piece {
                        let piece_char = match (piece.piece_type, piece.color) {
                            (PieceType::Pawn, Color::White) => "♙",
                            (PieceType::Knight, Color::White) => "♘",
                            (PieceType::Bishop, Color::White) => "♗",
                            (PieceType::Rook, Color::White) => "♖",
                            (PieceType::Queen, Color::White) => "♕",
                            (PieceType::King, Color::White) => "♔",
                            (PieceType::Pawn, Color::Black) => "♟",
                            (PieceType::Knight, Color::Black) => "♞",
                            (PieceType::Bishop, Color::Black) => "♝",
                            (PieceType::Rook, Color::Black) => "♜",
                            (PieceType::Queen, Color::Black) => "♛",
                            (PieceType::King, Color::Black) => "♚",
                        };

                        let text_color = if piece.color == Color::White {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        };

                        ui.put(
                            response.rect,
                            egui::Label::new(
                                egui::RichText::new(piece_char)
                                    .size(square_size * 0.8)
                                    .color(text_color)
                            )
                        );
                    }

                    if response.clicked() {
                        if let Some(from) = self.selected_square {
                            if self.board.is_valid_move(from, pos) {
                                self.board.make_move(from, pos);
                            }
                            self.selected_square = None;
                        } else {
                            self.selected_square = Some(pos);
                        }
                    }
                }
            });
        }

        // Reset hover state at the end of the frame
        if !ui.input(|i| i.pointer.primary_down()) {
            self.hovered_square = None;
        }

        // Draw game status
        ui.horizontal(|ui| {
            let current_turn = self.board.get_current_turn();
            ui.label(format!("Current turn: {}", 
                if current_turn == Color::White { "White" } else { "Black" }));
            
            if self.board.is_in_check(current_turn) {
                if self.board.is_checkmate(current_turn) {
                    ui.label(" - Checkmate!");
                } else {
                    ui.label(" - Check!");
                }
            }
        });

        // Draw move history
        if let Some(last_move) = self.board.get_last_move() {
            ui.label(format!(
                "Last move: {} from {:?} to {:?}{}{}{}",
                match last_move.piece.piece_type {
                    PieceType::Pawn => "Pawn",
                    PieceType::Knight => "Knight",
                    PieceType::Bishop => "Bishop",
                    PieceType::Rook => "Rook",
                    PieceType::Queen => "Queen",
                    PieceType::King => "King",
                },
                last_move.from,
                last_move.to,
                if last_move.captured.is_some() { " (capture)" } else { "" },
                if last_move.castle { " (castle)" } else { "" },
                if last_move.en_passant { " (en passant)" } else { "" },
            ));
        }

        // Show promotion dialog if needed
        if let Some(pos) = self.promotion_dialog {
            egui::Window::new("Promote Pawn")
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Queen").clicked() {
                            self.promote_pawn(pos, PieceType::Queen);
                        }
                        if ui.button("Rook").clicked() {
                            self.promote_pawn(pos, PieceType::Rook);
                        }
                        if ui.button("Bishop").clicked() {
                            self.promote_pawn(pos, PieceType::Bishop);
                        }
                        if ui.button("Knight").clicked() {
                            self.promote_pawn(pos, PieceType::Knight);
                        }
                    });
                });
        }
    }

    fn promote_pawn(&mut self, pos: (u8, u8), piece_type: PieceType) {
        let color = if pos.1 == 7 { Color::White } else { Color::Black };
        self.board.set_piece(pos, Some(Piece {
            piece_type,
            color,
        }));
        self.promotion_dialog = None;
    }
}

impl eframe::App for ChessGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_board(ui);
        });
    }
} 
use eframe::egui;
use crate::chess::{Board, Color, Piece, PieceType};
use crate::verify::verify_moves;

pub struct ChessGui {
    board: Board,
    selected_square: Option<(u8, u8)>,
    promotion_dialog: Option<(u8, u8)>, // Coordinates of pawn being promoted
    hovered_square: Option<(u8, u8)>, // Add this field
    game_over: bool, // Add this field
}

impl ChessGui {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            selected_square: None,
            promotion_dialog: None,
            hovered_square: None, // Initialize new field
            game_over: false, // Initialize new field
        }
    }

    fn reset_game(&mut self) {
        self.board = Board::new();
        self.selected_square = None;
        self.promotion_dialog = None;
        self.hovered_square = None;
        self.game_over = false;
    }

    fn draw_board(&mut self, ui: &mut egui::Ui) {
        let board_size = ui.available_width().min(ui.available_height());
        let square_size = board_size / 8.0;

        for rank in 0..8 {
            ui.horizontal(|ui| {
                for file in 0..8 {
                    let pos = (file as u8, rank as u8);
                    let is_dark = (rank + file) % 2 == 1;
                    
                    // Base square color
                    let base_color = if is_dark {
                        egui::Color32::from_rgb(181, 136, 99)
                    } else {
                        egui::Color32::from_rgb(240, 217, 181)
                    };

                    let piece = self.board.get_piece(pos);
                    
                    // Determine if this square should be highlighted
                    let is_valid_move = if let Some(selected_pos) = self.selected_square {
                        // Show valid moves for selected piece
                        self.board.is_valid_move(selected_pos, pos)
                    } else if let Some(hover_pos) = self.hovered_square {
                        // Show valid moves for hovered piece
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

                    // Square coloring logic
                    let square_color = if Some(pos) == self.selected_square {
                        egui::Color32::from_rgb(100, 100, 255) // Selected piece highlight
                    } else if is_valid_move {
                        if piece.is_some() {
                            egui::Color32::from_rgb(255, 50, 50) // Capture highlight (brighter red)
                        } else {
                            egui::Color32::from_rgb(50, 255, 50) // Move highlight (brighter green)
                        }
                    } else if Some(pos) == self.hovered_square {
                        if let Some(hover_piece) = self.board.get_piece(pos) {
                            if hover_piece.color == self.board.get_current_turn() {
                                egui::Color32::from_rgb(180, 180, 255) // Hover highlight for current player's pieces
                            } else {
                                base_color
                            }
                        } else {
                            base_color
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

                    // Update hover state before checking valid moves
                    if ui.input(|i| i.pointer.hover_pos().is_some()) {
                        if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                            let rect = response.rect;
                            if rect.contains(hover_pos) {
                                self.hovered_square = Some(pos);
                            }
                        }
                    }

                    // Draw piece
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

                    // Handle clicks
                    if response.clicked() {
                        if let Some(from) = self.selected_square {
                            if self.board.is_valid_move(from, pos) {
                                if self.board.needs_promotion(from, pos) {
                                    self.promotion_dialog = Some(pos);
                                    self.selected_square = Some(from);
                                } else {
                                    self.board.make_move(from, pos);
                                    self.selected_square = None;
                                }
                            } else {
                                // Clear selection if clicking on an invalid move
                                self.selected_square = None;
                                // If clicking on own piece, select it
                                if let Some(piece) = self.board.get_piece(pos) {
                                    if piece.color == self.board.get_current_turn() {
                                        self.selected_square = Some(pos);
                                    }
                                }
                            }
                        } else {
                            if let Some(piece) = self.board.get_piece(pos) {
                                if piece.color == self.board.get_current_turn() {
                                    self.selected_square = Some(pos);
                                }
                            }
                        }
                    }
                }
            });
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

        // Check for checkmate after drawing the board
        if self.board.is_checkmate(self.board.get_current_turn()) {
            self.game_over = true;
        }

        // Show game over dialog
        if self.game_over {
            let winner = if self.board.get_current_turn() == Color::White {
                "Black"
            } else {
                "White"
            };

            egui::Window::new("Game Over")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(format!("{} Wins!", winner));
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let available_width = ui.available_width();
                            ui.add_space(available_width / 4.0);
                            if ui.button("Play Again").clicked() {
                                self.reset_game();
                            }
                            ui.add_space(8.0);
                            if ui.button("Quit").clicked() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.add_space(available_width / 4.0);
                        });
                    });
                });

            // Verify all moves
            let move_history = self.board.get_move_history();
            match verify_moves(&move_history) {
                Ok(true) => {
                    ui.label("✓ All moves verified with zero-knowledge proof!");
                }
                Ok(false) => {
                    ui.label("❌ Move verification failed - proof invalid");
                }
                Err(e) => {
                    ui.label(format!("❌ Error creating proof: {}", e));
                }
            }
        }
    }

    fn promote_pawn(&mut self, pos: (u8, u8), piece_type: PieceType) {
        if let Some(from) = self.selected_square {
            let color = if pos.1 == 7 { Color::White } else { Color::Black };
            if self.board.make_move(from, pos) {
                self.board.set_piece(pos, Some(Piece {
                    piece_type,
                    color,
                }));
            }
        }
        self.promotion_dialog = None;
        self.selected_square = None;
    }
}

impl eframe::App for ChessGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_board(ui);
        });
    }
} 
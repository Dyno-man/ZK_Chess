mod chess;
mod gui;
mod verify;

use gui::ChessGui;
use eframe::egui;
use chess::{Board, MoveData};
use verify::verify_moves;
use std::env;

fn main() {
    // Check command line arguments
    let args: Vec<String> = env::args().collect();
    
    // If "test" argument is provided, run the test program
    if args.len() > 1 && args[1] == "test" {
        run_test_program();
    } else {
        // Otherwise, run the GUI
        run_gui();
    }
}

fn run_test_program() {
    println!("Running test program...");
    
    // Create a test board and make some moves
    let mut board = Board::new();
    
    // Make some moves
    board.make_move((6, 1), (6, 3)); // White pawn e2-e4
    board.make_move((4, 6), (4, 4)); // Black pawn e7-e5
    board.make_move((5, 1), (5, 3)); // White pawn f2-f4
    board.make_move((3, 7), (7, 3)); // Black queen d8-h4
    
    // Get the move history
    let move_history = board.get_move_history();
    
    // Print the move history
    println!("Move history:");
    for (i, m) in move_history.iter().enumerate() {
        println!("Move {}: from ({}, {}) to ({}, {}), piece: {}, color: {}", 
                i, m.from_x, m.from_y, m.to_x, m.to_y, m.piece_type, m.piece_color);
    }
    
    // Verify the moves
    let result = verify_moves(&move_history);
    
    // Print the result
    match result {
        Ok(true) => println!("All moves verified successfully!"),
        Ok(false) => println!("Move verification failed - proof invalid"),
        Err(e) => println!("Error verifying moves: {}", e),
    }
}

fn run_gui() {
    println!("Launching chess GUI...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([855.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "ZK Chess",
        options,
        Box::new(|_cc| Box::new(ChessGui::new())),
    ).unwrap();
} 
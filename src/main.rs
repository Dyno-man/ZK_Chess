mod chess;
mod gui;

use gui::ChessGui;
use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "ZK Chess",
        options,
        Box::new(|_cc| Box::new(ChessGui::new())),
    ).unwrap();
} 
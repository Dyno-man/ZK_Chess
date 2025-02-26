# ZK Chess

A chess game implementation in Rust that uses Zero-Knowledge proofs to verify the validity of chess games.

## Features

- Full chess game implementation with all standard rules
- Interactive GUI using egui
- Move validation for all pieces
- Special moves support:
  - Castling
  - En passant
  - Pawn promotion
- Zero-Knowledge proof generation and verification using Noir
- Board state hashing for tamper-proof game recording

## Technical Stack

- **Frontend**: Rust with egui framework
- **Chess Logic**: Pure Rust implementation
- **ZK Proofs**: Noir (for move validation circuits)
- **Cryptography**: SHA-256 for board state hashing

## Prerequisites

- Rust (latest stable version)
- Nargo (Noir's package manager)
- Cargo

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Dyno-man/ZK_Chess
cd ZK_Chess
```

2. Build the project:
```bash
cargo build --release
```

3. Run the game:
```bash
cargo run --release
```

## How It Works

1. The game records every move made during play
2. Each move is validated in real-time for legal chess moves
3. When a game ends, a Zero-Knowledge proof is generated that proves:
   - All moves were legal chess moves
   - Moves were made in the correct sequence
   - The final board state is valid
4. The proof can be verified without revealing the specific moves made

## Project Structure

- `src/` - Main Rust game implementation
  - `main.rs` - Entry point
  - `chess.rs` - Chess logic and move validation
  - `gui.rs` - Game interface using egui
  - `verify.rs` - ZK proof generation and verification
- `chess_circuit/` - Noir circuit implementation
  - `src/main.nr` - Move validation circuit
  - `src/test.nr` - Circuit tests

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Apahce 2.0

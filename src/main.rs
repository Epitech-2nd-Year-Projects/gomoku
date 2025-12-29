mod game;
mod protocol;

use crate::game::GameState;
use crate::protocol::{parse_command, Command, Response};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();
    let mut game = GameState::new();

    while let Some(line) = lines.next() {
        match line {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                if game.is_receiving_board_data() {
                    if input == "DONE" {
                        let response = game.handle_command(Command::Done);
                        if let Response::None = response {
                        } else {
                            if let Err(e) = writeln!(stdout, "{}", response) {
                                eprintln!("Failed to write to stdout: {}", e);
                                break;
                            }
                            if let Err(e) = stdout.flush() {
                                eprintln!("Failed to flush stdout: {}", e);
                                break;
                            }
                        }
                    } else {
                        game.handle_board_data(input);
                    }
                } else {
                    let command = parse_command(input);

                    if command == Command::End {
                        break;
                    }

                    let response = game.handle_command(command);

                    if let Response::None = response {
                    } else {
                        if let Err(e) = writeln!(stdout, "{}", response) {
                            eprintln!("Failed to write to stdout: {}", e);
                            break;
                        }
                        if let Err(e) = stdout.flush() {
                            eprintln!("Failed to flush stdout: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

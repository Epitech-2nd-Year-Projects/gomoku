mod game;
mod protocol;

use protocol::{parse_line, Command};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();
    let mut game = crate::game::GameState::new();

    while let Some(line) = lines.next() {
        match line {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                let command = parse_line(input);
                match command {
                    Command::Start(size) => {
                        println!("{}", game.handle_start(size));
                    }
                    Command::Turn(x, y) => {
                        println!("{}", game.handle_turn(x, y));
                    }
                    Command::Begin => {
                        println!("{}", game.handle_begin());
                    }
                    Command::Board => {
                        if game.handle_board_start() {
                            while let Some(board_line) = lines.next() {
                                match board_line {
                                    Ok(content) => {
                                        let content = content.trim();
                                        if content == "DONE" {
                                            println!("{}", game.handle_board_done());
                                            break;
                                        }

                                        let parts: Vec<&str> = content.split(',').collect();
                                        if parts.len() == 3 {
                                            if let (Ok(x), Ok(y), Ok(field)) = (
                                                parts[0].parse::<usize>(),
                                                parts[1].parse::<usize>(),
                                                parts[2].parse::<usize>(),
                                            ) {
                                                game.handle_board_move(x, y, field);
                                            } else {
                                                eprintln!("Error: Failed to parse integers in board line '{}'", content);
                                            }
                                        } else {
                                            eprintln!(
                                                "Error: Invalid format for board line '{}'",
                                                content
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error reading board line: {}", e);
                                        break;
                                    }
                                }
                            }
                        } else {
                            println!("ERROR game not initialized");
                        }
                    }
                    Command::Info(_, _) => {
                        // Ignore INFO commands for now
                    }
                    Command::About => {
                        println!("name=\"pbrain-brainrot\", version=\"1.0.0\", author=\"Brainrot\", country=\"FR\"");
                    }
                    Command::Restart => {
                        println!("{}", game.handle_restart());
                    }
                    Command::End => {
                        break;
                    }
                    Command::Error(msg) => {
                        println!("ERROR {}", msg);
                    }
                    Command::Unknown(msg) => {
                        println!("UNKNOWN {}", msg);
                    }
                }

                if let Err(e) = stdout.flush() {
                    eprintln!("Failed to flush stdout: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

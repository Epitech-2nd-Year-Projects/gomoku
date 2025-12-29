mod protocol;

use protocol::{parse_line, Command};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(line) = lines.next() {
        match line {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                let command = parse_line(input);
                match command {
                    Command::Start(_) => {
                        println!("OK");
                    }
                    Command::Turn(_, _) => {
                        // TODO: Implement actual move logic
                        println!("10,10");
                    }
                    Command::Begin => {
                        // TODO: Implement actual move logic
                        println!("10,10");
                    }
                    Command::Board => {
                        while let Some(board_line) = lines.next() {
                            match board_line {
                                Ok(content) => {
                                    let content = content.trim();
                                    if content == "DONE" {
                                        break;
                                    }
                                    // TODO: Parse board content
                                }
                                Err(e) => {
                                    eprintln!("Error reading board line: {}", e);
                                    break;
                                }
                            }
                        }
                        // TODO: Implement actual move logic
                        println!("10,10");
                    }
                    Command::Info(_, _) => {
                        // Ignore INFO commands for now
                    }
                    Command::About => {
                        println!("name=\"Brainrot\", version=\"1.0.0\", author=\"Brainrot\", country=\"FR\"");
                    }
                    Command::Restart => {
                        println!("OK");
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

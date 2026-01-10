mod board;
mod game;
mod protocol;
mod zobrist;

use crate::game::GameState;
use protocol::{parse_board_line, parse_line, BoardLine, Command};
use std::io::{self, BufRead, Write};
use std::panic;

fn handle_board_section<I>(lines: &mut I, game: &mut GameState) -> String
where
    I: Iterator<Item = Result<String, io::Error>>,
{
    let mut error: Option<String> = None;
    let mut done_received = false;

    if let Err(err) = game.handle_board_start() {
        error = Some(err.to_string());
    }

    for board_line in lines {
        match board_line {
            Ok(content) => {
                let content = content.trim();
                if content.is_empty() {
                    continue;
                }
                match parse_board_line(content) {
                    Ok(BoardLine::Done) => {
                        done_received = true;
                        break;
                    }
                    Ok(BoardLine::Move { x, y, field }) => {
                        if error.is_none() {
                            if let Err(err) = game.handle_board_move(x, y, field) {
                                error = Some(err.to_string());
                            }
                        }
                    }
                    Err(err) => {
                        if error.is_none() {
                            error = Some(format!("ERROR {}", err));
                        }
                    }
                }
            }
            Err(err) => {
                error = Some(format!("ERROR reading board line: {}", err));
                break;
            }
        }
    }

    if !done_received && error.is_none() {
        error = Some("ERROR missing DONE for BOARD".to_string());
    }

    match error {
        Some(msg) => msg,
        None => game.handle_board_done(),
    }
}

fn process_command(
    command: Command,
    lines: &mut impl Iterator<Item = Result<String, io::Error>>,
    game: &mut GameState,
) -> Option<String> {
    match command {
        Command::Start(size) => Some(game.handle_start(size)),
        Command::Turn(x, y) => Some(game.handle_turn(x, y)),
        Command::Begin => Some(game.handle_begin()),
        Command::Board => Some(handle_board_section(lines, game)),
        Command::Info(_, _) => None,
        Command::About => Some(
            "name=\"pbrain-brainrot\", version=\"1.0.0\", author=\"Brainrot\", country=\"FR\""
                .to_string(),
        ),
        Command::Restart => Some(game.handle_restart()),
        Command::End => None,
        Command::Error(msg) => Some(format!("ERROR {}", msg)),
        Command::Unknown(msg) => Some(format!("UNKNOWN {}", msg)),
    }
}

fn main() {
    panic::set_hook(Box::new(|_| {}));

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

                let command = parse_line(input);
                let is_end = matches!(command, Command::End);
                let needs_move_response = matches!(
                    command,
                    Command::Turn(_, _) | Command::Begin | Command::Board
                );

                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    process_command(command, &mut lines, &mut game)
                }));

                match result {
                    Ok(Some(response)) => println!("{}", response),
                    Ok(None) => {}
                    Err(_) => {
                        if needs_move_response {
                            let fallback = game.emergency_move();
                            println!("{}", fallback);
                        } else {
                            println!("ERROR internal error");
                        }
                    }
                }

                if is_end {
                    break;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_board_section_success() {
        let mut game = GameState::new();
        game.handle_start(20);

        let mut lines = vec![Ok("10,10,2".to_string()), Ok("DONE".to_string())].into_iter();

        let response = handle_board_section(&mut lines, &mut game);
        assert!(!response.contains("ERROR"));

        let parts: Vec<&str> = response.split(',').collect();
        assert_eq!(parts.len(), 2);
        let x: usize = parts[0].parse().unwrap();
        let y: usize = parts[1].parse().unwrap();
        assert!(x < 20 && y < 20);
        assert_ne!((x, y), (10, 10));
    }

    #[test]
    fn test_handle_board_section_missing_done() {
        let mut game = GameState::new();
        game.handle_start(20);

        let mut lines = vec![Ok("10,10,2".to_string())].into_iter();
        let response = handle_board_section(&mut lines, &mut game);

        assert_eq!(response, "ERROR missing DONE for BOARD");
    }

    #[test]
    fn test_handle_board_section_parse_error() {
        let mut game = GameState::new();
        game.handle_start(20);

        let mut lines = vec![Ok("bad".to_string()), Ok("DONE".to_string())].into_iter();
        let response = handle_board_section(&mut lines, &mut game);

        assert_eq!(response, "ERROR Invalid BOARD line 'bad'");
    }

    #[test]
    fn test_handle_board_section_auto_initializes() {
        let mut game = GameState::new();
        let mut lines = vec![Ok("DONE".to_string())].into_iter();
        let response = handle_board_section(&mut lines, &mut game);

        assert!(!response.contains("ERROR"));
        let parts: Vec<&str> = response.split(',').collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_handle_board_section_io_error() {
        let mut game = GameState::new();
        game.handle_start(20);

        let mut lines = vec![Err(io::Error::new(io::ErrorKind::Other, "boom"))].into_iter();
        let response = handle_board_section(&mut lines, &mut game);

        assert_eq!(response, "ERROR reading board line: boom");
    }
}

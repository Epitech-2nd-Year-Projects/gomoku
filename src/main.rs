mod board;
mod debug;
mod game;
mod protocol;

use crate::game::GameState;
use protocol::{parse_board_line, parse_line, BoardLine, Command};
use std::io::{self, BufRead, Write};

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

                let command = parse_line(input);
                crate::debug!("Received command: {:?}", command);
                match command {
                    Command::Start(size) => {
                        crate::debug!("Starting game with size: {}", size);
                        println!("{}", game.handle_start(size));
                    }
                    Command::Turn(x, y) => {
                        crate::debug!("Opponent played: {},{}", x, y);
                        let response = game.handle_turn(x, y);
                        crate::debug!("Bot response: {}", response);
                        println!("{}", response);
                    }
                    Command::Begin => {
                        crate::debug!("Begin command received");
                        println!("{}", game.handle_begin());
                    }
                    Command::Board => {
                        crate::debug!("Board command received");
                        let response = handle_board_section(&mut lines, &mut game);
                        crate::debug!("Bot response to board: {}", response);
                        println!("{}", response);
                    }
                    Command::Info(_, _) => {
                        // Ignore INFO commands for now
                    }
                    Command::About => {
                        println!("name=\"pbrain-brainrot\", version=\"1.0.0\", author=\"Brainrot\", country=\"FR\"");
                    }
                    Command::Restart => {
                        crate::debug!("Restart command received");
                        println!("{}", game.handle_restart());
                    }
                    Command::End => {
                        crate::debug!("End command received, exiting");
                        break;
                    }
                    Command::Error(msg) => {
                        crate::debug!("Error command: {}", msg);
                        println!("ERROR {}", msg);
                    }
                    Command::Unknown(msg) => {
                        crate::debug!("Unknown command: {}", msg);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_board_section_success() {
        let mut game = GameState::new();
        game.handle_start(20);

        let mut lines = vec![Ok("10,10,2".to_string()), Ok("DONE".to_string())].into_iter();

        let response = handle_board_section(&mut lines, &mut game);
        assert_eq!(response, "0,0");
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
    fn test_handle_board_section_uninitialized() {
        let mut game = GameState::new();
        let mut lines = vec![Ok("DONE".to_string())].into_iter();
        let response = handle_board_section(&mut lines, &mut game);

        assert_eq!(response, "ERROR game not initialized");
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

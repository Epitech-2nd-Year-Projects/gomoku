#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Start(usize),
    Turn(usize, usize),
    Begin,
    Board,
    Info(String, String),
    End,
    About,
    Restart,
    Error(String),
    Unknown(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoardLine {
    Move { x: usize, y: usize, field: usize },
    Done,
}

pub fn parse_line(line: &str) -> Command {
    let line = line.trim();
    if line.is_empty() {
        return Command::Unknown(String::new());
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    let raw_command = parts[0];
    let command = raw_command.to_ascii_uppercase();

    match command.as_str() {
        "START" => {
            if parts.len() >= 2 {
                if let Ok(size) = parts[1].parse::<usize>() {
                    Command::Start(size)
                } else {
                    Command::Error(format!("Invalid size for START: {}", parts[1]))
                }
            } else {
                Command::Error("Missing size for START".to_string())
            }
        }
        "TURN" => {
            if parts.len() >= 2 {
                let coords_str = parts[1..].join("");
                parse_coordinates(&coords_str)
                    .map(|(x, y)| Command::Turn(x, y))
                    .unwrap_or_else(|_| {
                        Command::Error(format!("Invalid coordinates for TURN: {}", coords_str))
                    })
            } else {
                Command::Error("Missing coordinates for TURN".to_string())
            }
        }
        "BEGIN" => Command::Begin,
        "BOARD" => Command::Board,
        "INFO" => {
            if parts.len() >= 3 {
                let value = parts[2..].join(" ");
                Command::Info(parts[1].to_string(), value)
            } else {
                Command::Error("Missing arguments for INFO".to_string())
            }
        }
        "END" => Command::End,
        "ABOUT" => Command::About,
        "RESTART" => Command::Restart,
        _ => Command::Unknown(raw_command.to_string()),
    }
}

fn parse_coordinates(s: &str) -> Result<(usize, usize), ()> {
    let s = s.replace(' ', "");
    let coords: Vec<&str> = s.split(',').filter(|p| !p.is_empty()).collect();
    if coords.len() < 2 {
        return Err(());
    }
    let x = coords[0].parse::<usize>().map_err(|_| ())?;
    let y = coords[1].parse::<usize>().map_err(|_| ())?;
    Ok((x, y))
}

pub fn parse_board_line(line: &str) -> Result<BoardLine, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err("Empty BOARD line".to_string());
    }
    if trimmed.eq_ignore_ascii_case("DONE") {
        return Ok(BoardLine::Done);
    }

    let normalized = trimmed.replace(' ', "");
    let parts: Vec<&str> = normalized.split(',').filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 {
        return Err(format!("Invalid BOARD line '{}'", trimmed));
    }

    let x = parts[0]
        .parse::<usize>()
        .map_err(|_| format!("Invalid BOARD line '{}'", trimmed))?;
    let y = parts[1]
        .parse::<usize>()
        .map_err(|_| format!("Invalid BOARD line '{}'", trimmed))?;
    let field = parts[2]
        .parse::<usize>()
        .map_err(|_| format!("Invalid BOARD line '{}'", trimmed))?;

    Ok(BoardLine::Move { x, y, field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_start() {
        assert_eq!(parse_line("START 20"), Command::Start(20));
        match parse_line("START invalid") {
            Command::Error(_) => assert!(true),
            _ => assert!(false, "Should be Error"),
        }
    }

    #[test]
    fn test_parse_turn() {
        assert_eq!(parse_line("TURN 10,11"), Command::Turn(10, 11));
        match parse_line("TURN 10,invalid") {
            Command::Error(_) => assert!(true),
            _ => assert!(false, "Should be Error"),
        }
    }

    #[test]
    fn test_parse_simple_commands() {
        assert_eq!(parse_line("BEGIN"), Command::Begin);
        assert_eq!(parse_line("BOARD"), Command::Board);
        assert_eq!(parse_line("END"), Command::End);
        assert_eq!(parse_line("ABOUT"), Command::About);
        assert_eq!(parse_line("RESTART"), Command::Restart);
    }

    #[test]
    fn test_parse_case_insensitive_commands() {
        assert_eq!(parse_line("start 20"), Command::Start(20));
        assert_eq!(parse_line("Start 20"), Command::Start(20));
        assert_eq!(parse_line("begin"), Command::Begin);
        assert_eq!(parse_line("tUrN 3,4"), Command::Turn(3, 4));
    }

    #[test]
    fn test_parse_info() {
        assert_eq!(
            parse_line("INFO timeout_turn 1000"),
            Command::Info("timeout_turn".to_string(), "1000".to_string())
        );
    }

    #[test]
    fn test_parse_unknown() {
        match parse_line("INVALID") {
            Command::Unknown(_) => assert!(true),
            _ => assert!(false, "Should be Unknown"),
        }
    }

    #[test]
    fn test_parse_board_line_move() {
        assert_eq!(
            parse_board_line("10,11,2"),
            Ok(BoardLine::Move {
                x: 10,
                y: 11,
                field: 2
            })
        );
        assert_eq!(
            parse_board_line(" 3, 4 , 1 "),
            Ok(BoardLine::Move {
                x: 3,
                y: 4,
                field: 1
            })
        );
    }

    #[test]
    fn test_parse_board_line_done() {
        assert_eq!(parse_board_line("DONE"), Ok(BoardLine::Done));
    }

    #[test]
    fn test_parse_board_line_invalid() {
        assert!(parse_board_line("10,11").is_err());
        assert!(parse_board_line("10,xx,1").is_err());
        assert!(parse_board_line("").is_err());
        assert!(parse_board_line("abc").is_err());
        assert!(parse_board_line("-1,0,1").is_err());
    }

    #[test]
    fn test_parse_board_line_extra_args_tolerated() {
        assert_eq!(
            parse_board_line("1,2,3,4"),
            Ok(BoardLine::Move {
                x: 1,
                y: 2,
                field: 3
            })
        );
    }

    #[test]
    fn test_parse_coordinates_with_spaces() {
        assert_eq!(parse_line("TURN 10 , 11"), Command::Turn(10, 11));
        assert_eq!(parse_line("TURN  10,11"), Command::Turn(10, 11));
        assert_eq!(parse_line("TURN 10,  11"), Command::Turn(10, 11));
    }

    #[test]
    fn test_parse_board_line_with_spaces() {
        assert_eq!(
            parse_board_line("10 , 11 , 2"),
            Ok(BoardLine::Move {
                x: 10,
                y: 11,
                field: 2
            })
        );
    }

    #[test]
    fn test_parse_extra_whitespace() {
        assert_eq!(parse_line("  START   20  "), Command::Start(20));
        assert_eq!(parse_line("  BEGIN  "), Command::Begin);
        assert_eq!(parse_line("\tTURN 5,5\t"), Command::Turn(5, 5));
    }

    #[test]
    fn test_parse_extra_args_ignored() {
        assert_eq!(parse_line("START 20 extra args"), Command::Start(20));
    }

    #[test]
    fn test_parse_done_case_insensitive() {
        assert_eq!(parse_board_line("done"), Ok(BoardLine::Done));
        assert_eq!(parse_board_line("Done"), Ok(BoardLine::Done));
        assert_eq!(parse_board_line("dOnE"), Ok(BoardLine::Done));
    }
}

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

pub fn parse_line(line: &str) -> Command {
    let line = line.trim();
    if line.is_empty() {
        return Command::Unknown("".to_string());
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    let command = parts[0];

    match command {
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
                parse_coordinates(parts[1])
                    .map(|(x, y)| Command::Turn(x, y))
                    .unwrap_or_else(|_| {
                        Command::Error(format!("Invalid coordinates for TURN: {}", parts[1]))
                    })
            } else {
                Command::Error("Missing coordinates for TURN".to_string())
            }
        }
        "BEGIN" => Command::Begin,
        "BOARD" => Command::Board,
        "INFO" => {
            if parts.len() >= 3 {
                Command::Info(parts[1].to_string(), parts[2].to_string())
            } else {
                Command::Error("Missing arguments for INFO".to_string())
            }
        }
        "END" => Command::End,
        "ABOUT" => Command::About,
        "RESTART" => Command::Restart,
        _ => Command::Unknown(format!("Unknown command: {}", command)),
    }
}

fn parse_coordinates(s: &str) -> Result<(usize, usize), ()> {
    let coords: Vec<&str> = s.split(',').collect();
    if coords.len() != 2 {
        return Err(());
    }
    let x = coords[0].parse::<usize>().map_err(|_| ())?;
    let y = coords[1].parse::<usize>().map_err(|_| ())?;
    Ok((x, y))
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
}

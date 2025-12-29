use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Command {
    Start(usize),
    Turn(usize, usize),
    Begin,
    Board,
    Info(String, String),
    End,
    About,
    Done,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Ok,
    Error(String),
    Unknown(String),
    Message(String),
    Move(usize, usize),
    Debug(String),
    Raw(String),
    None,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "OK"),
            Response::Error(msg) => write!(f, "ERROR {}", msg),
            Response::Unknown(msg) => write!(f, "UNKNOWN {}", msg),
            Response::Message(msg) => write!(f, "MESSAGE {}", msg),
            Response::Move(x, y) => write!(f, "{},{}", x, y),
            Response::Debug(msg) => write!(f, "DEBUG {}", msg),
            Response::Raw(msg) => write!(f, "{}", msg),
            Response::None => Ok(()),
        }
    }
}

pub fn parse_command(line: &str) -> Command {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Command::Unknown;
    }

    match parts[0] {
        "START" => {
            if parts.len() < 2 {
                return Command::Unknown;
            }
            if let Ok(size) = parts[1].parse::<usize>() {
                Command::Start(size)
            } else {
                Command::Unknown
            }
        }
        "TURN" => {
            if parts.len() < 2 {
                return Command::Unknown;
            }
            parse_coordinate(parts[1])
                .map(|(x, y)| Command::Turn(x, y))
                .unwrap_or(Command::Unknown)
        }
        "BEGIN" => Command::Begin,
        "BOARD" => Command::Board,
        "INFO" => {
            if parts.len() < 3 {
                Command::Unknown
            } else {
                Command::Info(parts[1].to_string(), parts[2].to_string())
            }
        }
        "END" => Command::End,
        "ABOUT" => Command::About,
        "DONE" => Command::Done,
        _ => Command::Unknown,
    }
}

pub fn parse_coordinate(coord: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = coord.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let x = parts[0].parse::<usize>().ok()?;
    let y = parts[1].parse::<usize>().ok()?;
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_start() {
        assert_eq!(parse_command("START 20"), Command::Start(20));
        assert_eq!(parse_command("START"), Command::Unknown);
        assert_eq!(parse_command("START foo"), Command::Unknown);
    }

    #[test]
    fn test_parse_turn() {
        assert_eq!(parse_command("TURN 10,11"), Command::Turn(10, 11));
        assert_eq!(parse_command("TURN 10"), Command::Unknown);
        assert_eq!(parse_command("TURN 10,foo"), Command::Unknown);
    }

    #[test]
    fn test_parse_info() {
        assert_eq!(
            parse_command("INFO timeout_match 3000"),
            Command::Info("timeout_match".to_string(), "3000".to_string())
        );
    }
}

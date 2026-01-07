use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn run_commands(commands: &[&str]) -> Vec<String> {
    let mut child = Command::new("./target/debug/pbrain-gomoku-ai")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    for cmd in commands {
        writeln!(stdin, "{}", cmd).unwrap();
    }
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    let reader = BufReader::new(&*output.stdout);
    reader.lines().map(|l| l.unwrap()).collect()
}

fn parse_coords(s: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let x = parts[0].parse().ok()?;
    let y = parts[1].parse().ok()?;
    Some((x, y))
}

fn is_valid_move(s: &str) -> bool {
    if s.starts_with("ERROR") || s.starts_with("UNKNOWN") {
        return false;
    }
    parse_coords(s).is_some()
}

#[test]
fn test_start_begin_sequence() {
    let responses = run_commands(&["START 20", "BEGIN"]);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    let (x, y) = parse_coords(&responses[1]).unwrap();
    assert!(x < 20 && y < 20);
}

#[test]
fn test_turn_sequence() {
    let responses = run_commands(&["START 20", "TURN 10,10"]);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    let (x, y) = parse_coords(&responses[1]).unwrap();
    assert!(x < 20 && y < 20);
    assert_ne!((x, y), (10, 10));
}

#[test]
fn test_multiple_turns() {
    let responses = run_commands(&["START 20", "TURN 0,0", "TURN 0,1"]);
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    assert!(is_valid_move(&responses[2]));
    let (x1, y1) = parse_coords(&responses[1]).unwrap();
    let (x2, y2) = parse_coords(&responses[2]).unwrap();
    assert_ne!((x1, y1), (x2, y2));
}

#[test]
fn test_board_prefilled_sequence() {
    let responses = run_commands(&[
        "START 20",
        "BOARD",
        "10,10,1",
        "10,11,2",
        "DONE",
    ]);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    let (x, y) = parse_coords(&responses[1]).unwrap();
    assert!(x < 20 && y < 20);
    assert_ne!((x, y), (10, 10));
    assert_ne!((x, y), (10, 11));
}

#[test]
fn test_board_then_turn() {
    let responses = run_commands(&[
        "START 20",
        "BOARD",
        "5,5,1",
        "DONE",
        "TURN 0,0",
    ]);
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    assert!(is_valid_move(&responses[2]));
}

#[test]
fn test_invalid_move_error() {
    let responses = run_commands(&["START 20", "TURN 20,20"]);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0], "OK");
    assert!(responses[1].starts_with("ERROR"));
}

#[test]
fn test_about_command() {
    let responses = run_commands(&["ABOUT"]);
    assert_eq!(responses.len(), 1);
    assert!(responses[0].contains("name="));
}

#[test]
fn test_restart_sequence() {
    let responses = run_commands(&["START 20", "TURN 0,0", "RESTART", "BEGIN"]);
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0], "OK");
    assert!(is_valid_move(&responses[1]));
    assert_eq!(responses[2], "OK");
    assert!(is_valid_move(&responses[3]));
}


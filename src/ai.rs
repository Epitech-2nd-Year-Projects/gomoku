use crate::board::{Board, Cell};

const SCORE_FOUR_OPEN: i32 = 10_000;
const SCORE_FOUR_BLOCKED: i32 = 1_000;
const SCORE_THREE_OPEN: i32 = 1_000;
const SCORE_THREE_BLOCKED: i32 = 100;
const SCORE_TWO_OPEN: i32 = 100;
const SCORE_TWO_BLOCKED: i32 = 10;

pub fn find_winning_move(board: &Board, player: Cell) -> Option<(usize, usize)> {
    let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
    let size = board.size();

    for y in 0..size {
        for x in 0..size {
            if !board.is_empty(x, y) {
                continue;
            }

            for &(dx, dy) in &directions {
                let mut count = 1;

                for step in 1..5 {
                    let nx = x as isize + dx * step;
                    let ny = y as isize + dy * step;

                    if nx < 0 || ny < 0 || nx as usize >= size || ny as usize >= size {
                        break;
                    }

                    if let Some(cell) = board.get_cell(nx as usize, ny as usize) {
                        if cell == player {
                            count += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                for step in 1..5 {
                    let nx = x as isize - dx * step;
                    let ny = y as isize - dy * step;

                    if nx < 0 || ny < 0 || nx as usize >= size || ny as usize >= size {
                        break;
                    }

                    if let Some(cell) = board.get_cell(nx as usize, ny as usize) {
                        if cell == player {
                            count += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if count >= 5 {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

pub fn find_blocking_move(board: &Board, opponent: Cell) -> Option<(usize, usize)> {
    find_winning_move(board, opponent)
}

fn evaluate_sequence(
    board: &Board,
    x: usize,
    y: usize,
    dx: isize,
    dy: isize,
    player: Cell,
    opponent: Cell,
) -> i32 {
    let mut right_count = 0;
    let mut blocked_right = false;
    let mut nx = x as isize + dx;
    let mut ny = y as isize + dy;

    for _ in 0..4 {
        if nx < 0 || ny < 0 {
            break;
        }
        if let Some(cell) = board.get_cell(nx as usize, ny as usize) {
            if cell == player {
                right_count += 1;
            } else if cell == opponent {
                blocked_right = true;
                break;
            } else {
                break;
            }
        } else {
            break;
        }
        nx += dx;
        ny += dy;
    }

    let mut left_count = 0;
    let mut blocked_left = false;
    nx = x as isize - dx;
    ny = y as isize - dy;

    for _ in 0..4 {
        if nx < 0 || ny < 0 {
            blocked_left = true;
            break;
        }
        if let Some(cell) = board.get_cell(nx as usize, ny as usize) {
            if cell == player {
                left_count += 1;
            } else if cell == opponent {
                blocked_left = true;
                break;
            } else {
                break;
            }
        } else {
            blocked_left = true;
            break;
        }
        nx -= dx;
        ny -= dy;
    }

    let total_count = left_count + right_count;
    let open_left = !blocked_left;
    let open_right = !blocked_right;

    match total_count {
        4 => {
            if open_left && open_right {
                SCORE_FOUR_OPEN
            } else if open_left || open_right {
                SCORE_FOUR_BLOCKED
            } else {
                0
            }
        }
        3 => {
            if open_left && open_right {
                SCORE_THREE_OPEN
            } else if open_left || open_right {
                SCORE_THREE_BLOCKED
            } else {
                0
            }
        }
        2 => {
            if open_left && open_right {
                SCORE_TWO_OPEN
            } else if open_left || open_right {
                SCORE_TWO_BLOCKED
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub fn evaluate(board: &Board, player: Cell) -> i32 {
    let opponent = match player {
        Cell::MyStone => Cell::OpStone,
        Cell::OpStone => Cell::MyStone,
        _ => return 0,
    };

    if board.check_five_in_a_row(player) {
        return 100_000;
    }
    if board.check_five_in_a_row(opponent) {
        return -100_000;
    }

    let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
    let mut score = 0;
    let size = board.size();

    for y in 0..size {
        for x in 0..size {
            if board.get_cell(x, y) == Some(player) {
                for &(dx, dy) in &directions {
                    score += evaluate_sequence(board, x, y, dx, dy, player, opponent);
                }
            } else if board.get_cell(x, y) == Some(opponent) {
                for &(dx, dy) in &directions {
                    score -= evaluate_sequence(board, x, y, dx, dy, opponent, player);
                }
            }
        }
    }

    score
}

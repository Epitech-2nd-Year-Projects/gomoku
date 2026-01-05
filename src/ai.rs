use crate::board::{Board, Cell};

const SCORE_WIN: i32 = 100_000;
const SCORE_FOUR_OPEN: i32 = 10_000;
const SCORE_FOUR_BLOCKED: i32 = 1_000;
const SCORE_THREE_OPEN: i32 = 1_000;
const SCORE_THREE_BLOCKED: i32 = 100;
const SCORE_TWO_OPEN: i32 = 100;
const SCORE_TWO_BLOCKED: i32 = 10;
const MAX_DEPTH: u8 = 4;

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
        return SCORE_WIN;
    }
    if board.check_five_in_a_row(opponent) {
        return -SCORE_WIN;
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

pub fn generate_moves(board: &Board) -> Vec<(usize, usize)> {
    let size = board.size();
    let center = size / 2;
    let mut moves = Vec::new();
    let mut prioritized = Vec::new();

    for y in 0..size {
        for x in 0..size {
            if !board.is_empty(x, y) {
                continue;
            }

            let distance_to_center = ((x as isize - center as isize).abs()
                + (y as isize - center as isize).abs()) as usize;

            let near_stone = (0..size)
                .flat_map(|dy| (0..size).map(move |dx| (dx, dy)))
                .filter(|&(nx, ny)| {
                    if nx == x && ny == y {
                        return false;
                    }
                    if board.get_cell(nx, ny) == Some(Cell::Empty) {
                        return false;
                    }
                    let dx = (nx as isize - x as isize).abs();
                    let dy = (ny as isize - y as isize).abs();
                    dx <= 2 && dy <= 2 && (dx > 0 || dy > 0)
                })
                .next()
                .is_some();

            if distance_to_center <= 3 || near_stone {
                prioritized.push((distance_to_center, x, y));
            } else {
                moves.push((x, y));
            }
        }
    }

    prioritized.sort_by_key(|&(dist, _, _)| dist);
    prioritized
        .into_iter()
        .map(|(_, x, y)| (x, y))
        .chain(moves.into_iter())
        .collect()
}

fn minimax(
    board: &mut Board,
    depth: u8,
    alpha: i32,
    beta: i32,
    maximizing: bool,
    player: Cell,
) -> i32 {
    if depth == 0 {
        return evaluate(board, player);
    }

    let opponent = match player {
        Cell::MyStone => Cell::OpStone,
        Cell::OpStone => Cell::MyStone,
        _ => return 0,
    };

    if board.check_five_in_a_row(player) {
        return SCORE_WIN + depth as i32;
    }
    if board.check_five_in_a_row(opponent) {
        return -SCORE_WIN - depth as i32;
    }
    if board.is_full() {
        return 0;
    }

    let current_player = if maximizing { player } else { opponent };
    let moves = generate_moves(board);

    if moves.is_empty() {
        return evaluate(board, player);
    }

    let mut alpha = alpha;
    let mut beta = beta;

    if maximizing {
        let mut max_eval = i32::MIN;

        for &(x, y) in moves.iter() {
            board.set_cell(x, y, current_player).unwrap();
            let eval = minimax(board, depth - 1, alpha, beta, false, player);
            board.set_cell(x, y, Cell::Empty).unwrap();

            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }

        max_eval
    } else {
        let mut min_eval = i32::MAX;

        for &(x, y) in moves.iter() {
            board.set_cell(x, y, current_player).unwrap();
            let eval = minimax(board, depth - 1, alpha, beta, true, player);
            board.set_cell(x, y, Cell::Empty).unwrap();

            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }

        min_eval
    }
}

pub fn find_best_move_minimax(board: &Board, player: Cell) -> Option<(usize, usize)> {
    let mut board_copy = *board;

    if let Some(move_pos) = find_winning_move(&board_copy, player) {
        return Some(move_pos);
    }

    if let Some(move_pos) = find_blocking_move(&board_copy, match player {
        Cell::MyStone => Cell::OpStone,
        Cell::OpStone => Cell::MyStone,
        _ => return None,
    }) {
        return Some(move_pos);
    }

    let moves = generate_moves(&board_copy);

    if moves.is_empty() {
        return None;
    }

    let mut best_move = None;
    let mut best_score = i32::MIN;

    for &(x, y) in moves.iter().take(15) {
        board_copy.set_cell(x, y, player).unwrap();
        let score = minimax(
            &mut board_copy,
            MAX_DEPTH - 1,
            i32::MIN,
            i32::MAX,
            false,
            player,
        );
        board_copy.set_cell(x, y, Cell::Empty).unwrap();

        if score > best_score {
            best_score = score;
            best_move = Some((x, y));
        }
    }

    best_move
}

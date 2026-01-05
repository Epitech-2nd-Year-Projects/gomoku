use crate::board::{Board, Cell};

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

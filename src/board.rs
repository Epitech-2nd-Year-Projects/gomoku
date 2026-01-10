use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Cell {
    #[default]
    Empty = 0,
    MyStone = 1,
    OpStone = 2,
    Forbidden = 3,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let char_rep = match self {
            Cell::Empty => '.',
            Cell::MyStone => 'X',
            Cell::OpStone => 'O',
            Cell::Forbidden => 'F',
        };
        write!(f, "{}", char_rep)
    }
}

#[derive(Clone, Copy)]
pub struct Board {
    cells: [Cell; 400],
    size: usize,
    hash: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [Cell::Empty; 400],
            size: 20,
            hash: 0,
        }
    }
}

impl Board {
    #[allow(dead_code)]
    pub fn new(size: usize) -> Option<Self> {
        if size != 20 {
            return None;
        }
        Some(Self::default())
    }

    #[inline]
    pub fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.size || y >= self.size {
            None
        } else {
            Some(y * self.size + x)
        }
    }

    #[inline]
    pub fn get_cell(&self, x: usize, y: usize) -> Option<Cell> {
        self.get_index(x, y).map(|idx| self.cells[idx])
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) -> Result<(), &'static str> {
        match self.get_index(x, y) {
            Some(idx) => {
                self.cells[idx] = cell;
                Ok(())
            }
            None => Err("Coordinates out of bounds"),
        }
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.get_cell(x, y) == Some(Cell::Empty)
    }

    pub fn clear(&mut self) {
        self.cells = [Cell::Empty; 400];
        self.hash = 0;
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    pub fn update_hash(&mut self, key: u64) {
        self.hash ^= key;
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.size).flat_map(move |y| (0..self.size).map(move |x| (x, y)))
    }

    pub fn iter_empty(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.iter_indices()
            .filter(move |&(x, y)| self.is_empty(x, y))
    }

    pub fn is_full(&self) -> bool {
        self.iter_empty().next().is_none()
    }

    pub fn check_five_in_a_row(&self, player: Cell) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for y in 0..self.size {
            for x in 0..self.size {
                if self.get_cell(x, y) != Some(player) {
                    continue;
                }

                for &(dx, dy) in &directions {
                    let mut count = 1;
                    for step in 1..5 {
                        let nx = x as isize + dx * step;
                        let ny = y as isize + dy * step;

                        if nx < 0 || ny < 0 {
                            break;
                        }

                        if self.get_cell(nx as usize, ny as usize) == Some(player) {
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    if count >= 5 {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board {{ size: {} }}", self.size)?;
        for y in 0..self.size {
            for x in 0..self.size {
                write!(f, "{} ", self.get_cell(x, y).unwrap())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board() {
        assert!(Board::new(20).is_some());
        assert!(Board::new(19).is_none());
    }

    #[test]
    fn test_indexing() {
        let board = Board::default();
        assert_eq!(board.get_index(0, 0), Some(0));
        assert_eq!(board.get_index(19, 0), Some(19));
        assert_eq!(board.get_index(0, 1), Some(20));
        assert_eq!(board.get_index(19, 19), Some(399));
        assert_eq!(board.get_index(20, 0), None);
        assert_eq!(board.get_index(0, 20), None);
    }

    #[test]
    fn test_cell_operations() {
        let mut board = Board::default();
        assert_eq!(board.get_cell(10, 10), Some(Cell::Empty));

        assert!(board.set_cell(10, 10, Cell::MyStone).is_ok());
        assert_eq!(board.get_cell(10, 10), Some(Cell::MyStone));

        assert!(board.set_cell(20, 20, Cell::MyStone).is_err());
    }

    #[test]
    fn test_iter_empty() {
        let mut board = Board::default();
        board.set_cell(0, 0, Cell::MyStone).unwrap();

        let empty_cells: Vec<_> = board.iter_empty().collect();
        assert_eq!(empty_cells.len(), 399);
        assert!(!empty_cells.contains(&(0, 0)));
    }

    #[test]
    fn test_check_five_in_a_row_horizontal() {
        let mut board = Board::default();
        for x in 0..5 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
        assert!(!board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_check_five_in_a_row_vertical() {
        let mut board = Board::default();
        for y in 0..5 {
            board.set_cell(0, y, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::OpStone));
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_check_five_in_a_row_diagonal() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(i, i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_check_five_in_a_row_anti_diagonal() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(i, 4 - i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_check_five_in_a_row_interrupted() {
        let mut board = Board::default();
        for x in 0..4 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        board.set_cell(5, 0, Cell::MyStone).unwrap();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_is_full() {
        let board = Board::default();
        assert!(!board.is_full());
    }

    #[test]
    fn test_indexing_all_corners() {
        let board = Board::default();
        assert_eq!(board.get_index(0, 0), Some(0));
        assert_eq!(board.get_index(19, 0), Some(19));
        assert_eq!(board.get_index(0, 19), Some(380));
        assert_eq!(board.get_index(19, 19), Some(399));
    }

    #[test]
    fn test_indexing_edge_row_0() {
        let board = Board::default();
        for x in 0..20 {
            assert_eq!(board.get_index(x, 0), Some(x));
        }
    }

    #[test]
    fn test_indexing_edge_row_19() {
        let board = Board::default();
        for x in 0..20 {
            assert_eq!(board.get_index(x, 19), Some(380 + x));
        }
    }

    #[test]
    fn test_indexing_edge_col_0() {
        let board = Board::default();
        for y in 0..20 {
            assert_eq!(board.get_index(0, y), Some(y * 20));
        }
    }

    #[test]
    fn test_indexing_edge_col_19() {
        let board = Board::default();
        for y in 0..20 {
            assert_eq!(board.get_index(19, y), Some(y * 20 + 19));
        }
    }

    #[test]
    fn test_indexing_out_of_bounds_regression() {
        let board = Board::default();
        assert_eq!(board.get_index(20, 0), None);
        assert_eq!(board.get_index(0, 20), None);
        assert_eq!(board.get_index(20, 20), None);
        assert_eq!(board.get_index(usize::MAX, 0), None);
        assert_eq!(board.get_index(0, usize::MAX), None);
    }

    #[test]
    fn test_cell_operations_at_corners() {
        let mut board = Board::default();

        let corners = [(0, 0), (19, 0), (0, 19), (19, 19)];
        for &(x, y) in &corners {
            assert!(board.is_empty(x, y));
            assert!(board.set_cell(x, y, Cell::MyStone).is_ok());
            assert_eq!(board.get_cell(x, y), Some(Cell::MyStone));
            assert!(!board.is_empty(x, y));
        }
    }

    #[test]
    fn test_cell_operations_at_edges() {
        let mut board = Board::default();

        for x in 0..20 {
            assert!(board.set_cell(x, 0, Cell::MyStone).is_ok());
            assert!(board.set_cell(x, 19, Cell::OpStone).is_ok());
        }
        for y in 1..19 {
            assert!(board.set_cell(0, y, Cell::MyStone).is_ok());
            assert!(board.set_cell(19, y, Cell::OpStone).is_ok());
        }

        for x in 0..20 {
            assert_eq!(board.get_cell(x, 0), Some(Cell::MyStone));
            assert_eq!(board.get_cell(x, 19), Some(Cell::OpStone));
        }
    }

    #[test]
    fn test_five_in_a_row_edge_row_0() {
        let mut board = Board::default();
        for x in 0..5 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_five_in_a_row_edge_row_19() {
        let mut board = Board::default();
        for x in 15..20 {
            board.set_cell(x, 19, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_five_in_a_row_edge_col_0() {
        let mut board = Board::default();
        for y in 0..5 {
            board.set_cell(0, y, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_five_in_a_row_edge_col_19() {
        let mut board = Board::default();
        for y in 15..20 {
            board.set_cell(19, y, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_five_in_a_row_diagonal_top_left_corner() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(i, i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_five_in_a_row_diagonal_bottom_right_corner() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(15 + i, 15 + i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_five_in_a_row_anti_diagonal_top_right_corner() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(19 - i, i, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_five_in_a_row_anti_diagonal_bottom_left_corner() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(4 - i, 15 + i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_no_false_positive_four_in_a_row() {
        let mut board = Board::default();
        for x in 0..4 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_no_false_positive_four_diagonal() {
        let mut board = Board::default();
        for i in 0..4 {
            board.set_cell(i, i, Cell::MyStone).unwrap();
        }
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_no_false_positive_four_anti_diagonal() {
        let mut board = Board::default();
        for i in 0..4 {
            board.set_cell(3 - i, i, Cell::MyStone).unwrap();
        }
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_no_false_positive_scattered_stones() {
        let mut board = Board::default();
        board.set_cell(0, 0, Cell::MyStone).unwrap();
        board.set_cell(5, 5, Cell::MyStone).unwrap();
        board.set_cell(10, 10, Cell::MyStone).unwrap();
        board.set_cell(15, 15, Cell::MyStone).unwrap();
        board.set_cell(19, 19, Cell::MyStone).unwrap();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_no_false_positive_wrong_player() {
        let mut board = Board::default();
        for x in 0..5 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
        assert!(!board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_multiple_win_lines_same_player() {
        let mut board = Board::default();
        for x in 0..5 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        for y in 0..5 {
            board.set_cell(10, y, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_both_players_win_lines() {
        let mut board = Board::default();
        for x in 0..5 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        for x in 0..5 {
            board.set_cell(x, 5, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
        assert!(board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_overline_six_in_a_row() {
        let mut board = Board::default();
        for x in 0..6 {
            board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_win_line_with_gap_no_win() {
        let mut board = Board::default();
        board.set_cell(0, 0, Cell::MyStone).unwrap();
        board.set_cell(1, 0, Cell::MyStone).unwrap();
        board.set_cell(3, 0, Cell::MyStone).unwrap();
        board.set_cell(4, 0, Cell::MyStone).unwrap();
        board.set_cell(5, 0, Cell::MyStone).unwrap();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_win_blocked_by_opponent() {
        let mut board = Board::default();
        board.set_cell(0, 0, Cell::MyStone).unwrap();
        board.set_cell(1, 0, Cell::MyStone).unwrap();
        board.set_cell(2, 0, Cell::OpStone).unwrap();
        board.set_cell(3, 0, Cell::MyStone).unwrap();
        board.set_cell(4, 0, Cell::MyStone).unwrap();
        board.set_cell(5, 0, Cell::MyStone).unwrap();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_diagonal_at_edge_boundary() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(i, 15 + i, Cell::MyStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::MyStone));
    }

    #[test]
    fn test_anti_diagonal_at_edge_boundary() {
        let mut board = Board::default();
        for i in 0..5 {
            board.set_cell(19 - i, 15 + i, Cell::OpStone).unwrap();
        }
        assert!(board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_empty_board_no_win() {
        let board = Board::default();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
        assert!(!board.check_five_in_a_row(Cell::OpStone));
    }

    #[test]
    fn test_single_stone_no_win() {
        let mut board = Board::default();
        board.set_cell(10, 10, Cell::MyStone).unwrap();
        assert!(!board.check_five_in_a_row(Cell::MyStone));
    }
}

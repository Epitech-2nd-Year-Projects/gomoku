use crate::board::{Board, Cell};

pub struct GameState {
    size: usize,
    is_initialized: bool,
    game_in_progress: bool,
    board: Board,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            size: 0,
            is_initialized: false,
            game_in_progress: false,
            board: Board::default(),
        }
    }

    pub fn handle_start(&mut self, size: usize) -> String {
        if size != 20 {
            return format!("ERROR unsupported size {}", size);
        }
        self.size = size;
        self.is_initialized = true;
        self.game_in_progress = false;
        self.board.clear();
        "OK".to_string()
    }

    pub fn handle_turn(&mut self, x: usize, y: usize) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        if x >= self.size || y >= self.size {
            return "ERROR coordinates out of range".to_string();
        }
        if !self.board.is_empty(x, y) {
            return "ERROR cell already occupied".to_string();
        }

        self.board.set_cell(x, y, Cell::OpStone).unwrap();
        self.game_in_progress = true;

        self.make_move()
    }

    pub fn handle_begin(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.game_in_progress = true;
        self.make_move()
    }

    pub fn handle_board_start(&mut self) -> bool {
        if !self.is_initialized {
            return false;
        }
        self.game_in_progress = true;
        self.board.clear();
        true
    }

    pub fn handle_board_move(&mut self, x: usize, y: usize, field: usize) {
        let cell = match field {
            1 => Cell::MyStone,
            2 => Cell::OpStone,
            3 => Cell::Forbidden,
            _ => Cell::Empty,
        };
        let _ = self.board.set_cell(x, y, cell);
    }

    pub fn handle_board_done(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.make_move()
    }

    pub fn handle_restart(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.game_in_progress = false;
        self.board.clear();
        "OK".to_string()
    }

    fn make_move(&mut self) -> String {
        // TODO: implement actual AI logic
        if self.board.is_empty(10, 10) {
            self.board.set_cell(10, 10, Cell::MyStone).unwrap();
            return "10,10".to_string();
        }

        let next_empty = self.board.iter_empty().next();
        if let Some((x, y)) = next_empty {
            self.board.set_cell(x, y, Cell::MyStone).unwrap();
            return format!("{},{}", x, y);
        }

        "ERROR board full".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let mut game = GameState::new();
        assert_eq!(game.handle_start(10), "ERROR unsupported size 10");
        assert_eq!(game.handle_start(20), "OK");
        assert!(game.is_initialized);
    }

    #[test]
    fn test_turn_handling() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert_eq!(game.handle_turn(20, 20), "ERROR coordinates out of range");

        let response = game.handle_turn(0, 0);
        assert!(!response.contains("ERROR"));
        assert_eq!(game.board.get_cell(0, 0), Some(Cell::OpStone));

        let parts: Vec<&str> = response.split(',').collect();
        let bot_x: usize = parts[0].parse().unwrap();
        let bot_y: usize = parts[1].parse().unwrap();
        assert_eq!(game.board.get_cell(bot_x, bot_y), Some(Cell::MyStone));

        assert_eq!(
            game.handle_turn(bot_x, bot_y),
            "ERROR cell already occupied"
        );
    }

    #[test]
    fn test_board_command() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.handle_board_start();
        game.handle_board_move(10, 10, 1);
        game.handle_board_move(10, 11, 2);

        assert_eq!(game.board.get_cell(10, 10), Some(Cell::MyStone));
        assert_eq!(game.board.get_cell(10, 11), Some(Cell::OpStone));

        let response = game.handle_board_done();
        assert!(!response.contains("ERROR"));
        assert_ne!(response, "10,10");
        assert_ne!(response, "10,11");
    }

    #[test]
    fn test_restart() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.handle_turn(0, 0);
        assert!(game.board.get_cell(0, 0).is_some());
        assert_ne!(game.board.get_cell(0, 0), Some(Cell::Empty));

        game.handle_restart();
        assert_eq!(game.board.get_cell(0, 0), Some(Cell::Empty));
        assert!(!game.game_in_progress);
    }
}

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

    pub fn validate_move(&self, x: usize, y: usize) -> Result<(), &'static str> {
        if !self.is_initialized {
            return Err("ERROR game not initialized");
        }
        if x >= self.size || y >= self.size {
            return Err("ERROR coordinates out of range");
        }
        if self.board.get_cell(x, y) == Some(Cell::Forbidden) {
            return Err("ERROR move forbidden");
        }
        if !self.board.is_empty(x, y) {
            return Err("ERROR cell already occupied");
        }
        Ok(())
    }

    pub fn handle_turn(&mut self, x: usize, y: usize) -> String {
        if let Err(e) = self.validate_move(x, y) {
            return e.to_string();
        }

        self.board.set_cell(x, y, Cell::OpStone).unwrap();
        self.game_in_progress = true;

        if self.game_over().is_some() {
            self.game_in_progress = false;
        }

        self.generate_move()
    }

    pub fn game_over(&self) -> Option<Cell> {
        if self.board.check_five_in_a_row(Cell::MyStone) {
            return Some(Cell::MyStone);
        }
        if self.board.check_five_in_a_row(Cell::OpStone) {
            return Some(Cell::OpStone);
        }
        if self.board.is_full() {
            return Some(Cell::Empty);
        }
        None
    }

    pub fn handle_begin(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.game_in_progress = true;
        self.generate_move()
    }

    pub fn handle_board_start(&mut self) -> Result<(), &'static str> {
        if !self.is_initialized {
            return Err("ERROR game not initialized");
        }
        self.game_in_progress = true;
        self.board.clear();
        Ok(())
    }

    pub fn handle_board_move(
        &mut self,
        x: usize,
        y: usize,
        field: usize,
    ) -> Result<(), &'static str> {
        if !self.is_initialized {
            return Err("ERROR game not initialized");
        }
        if x >= self.size || y >= self.size {
            return Err("ERROR coordinates out of range");
        }

        let cell = match field {
            0 => Cell::Empty,
            1 => Cell::MyStone,
            2 => Cell::OpStone,
            3 => Cell::Forbidden,
            _ => return Err("ERROR invalid board field"),
        };

        self.board
            .set_cell(x, y, cell)
            .map_err(|_| "ERROR coordinates out of range")
    }

    pub fn handle_board_done(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.generate_move()
    }

    pub fn handle_restart(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.game_in_progress = false;
        self.board.clear();
        "OK".to_string()
    }

    fn find_immediate_win(&mut self, player: Cell) -> Option<(usize, usize)> {
        let empty_cells: Vec<(usize, usize)> = self.board.iter_empty().collect();

        for (x, y) in empty_cells {
            self.board
                .set_cell(x, y, player)
                .expect("board indices from iter_empty");
            let is_win = self.board.check_five_in_a_row(player);
            self.board
                .set_cell(x, y, Cell::Empty)
                .expect("board indices from iter_empty");

            if is_win {
                return Some((x, y));
            }
        }

        None
    }

    fn generate_move(&mut self) -> String {
        let move_coords = self
            .find_immediate_win(Cell::MyStone)
            .or_else(|| self.find_immediate_win(Cell::OpStone))
            .or_else(|| self.fallback_move());

        if let Some((x, y)) = move_coords {
            self.board.set_cell(x, y, Cell::MyStone).unwrap();

            if self.game_over().is_some() {
                self.game_in_progress = false;
            }

            return format!("{},{}", x, y);
        }

        "ERROR board full".to_string()
    }

    fn fallback_move(&self) -> Option<(usize, usize)> {
        let center = self.size / 2;
        if self.validate_move(center, center).is_ok() {
            return Some((center, center));
        }

        self.board
            .iter_empty()
            .find(|&(x, y)| self.validate_move(x, y).is_ok())
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
    fn test_validate_move() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert!(game.validate_move(10, 10).is_ok());

        assert!(game.validate_move(20, 10).is_err());
        assert!(game.validate_move(10, 20).is_err());

        game.board.set_cell(10, 10, Cell::MyStone).unwrap();
        assert!(game.validate_move(10, 10).is_err());
        assert_eq!(
            game.validate_move(10, 10),
            Err("ERROR cell already occupied")
        );

        game.board.set_cell(11, 11, Cell::Forbidden).unwrap();
        assert!(game.validate_move(11, 11).is_err());
        assert_eq!(game.validate_move(11, 11), Err("ERROR move forbidden"));
    }

    #[test]
    fn test_fallback_move_center_then_first_empty() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert_eq!(game.fallback_move(), Some((10, 10)));

        game.board.set_cell(10, 10, Cell::OpStone).unwrap();
        assert_eq!(game.fallback_move(), Some((0, 0)));
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

        assert!(game.handle_board_start().is_ok());
        game.handle_board_move(10, 10, 1).unwrap();
        game.handle_board_move(10, 11, 2).unwrap();

        assert_eq!(game.board.get_cell(10, 10), Some(Cell::MyStone));
        assert_eq!(game.board.get_cell(10, 11), Some(Cell::OpStone));

        let response = game.handle_board_done();
        assert!(!response.contains("ERROR"));
        assert_ne!(response, "10,10");
        assert_ne!(response, "10,11");
    }

    #[test]
    fn test_board_move_errors() {
        let mut game = GameState::new();
        assert_eq!(game.handle_board_start(), Err("ERROR game not initialized"));
        assert_eq!(
            game.handle_board_move(0, 0, 1),
            Err("ERROR game not initialized")
        );

        game.handle_start(20);

        assert_eq!(
            game.handle_board_move(20, 0, 1),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.handle_board_move(0, 20, 1),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.handle_board_move(0, 0, 9),
            Err("ERROR invalid board field")
        );
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

    #[test]
    fn test_game_over_win() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..5 {
            game.board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));

        game.handle_restart();
        game.handle_start(20);

        for x in 0..5 {
            game.board.set_cell(x, 0, Cell::OpStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::OpStone));
    }

    #[test]
    fn test_game_over_draw() {
        let mut game = GameState::new();
        game.handle_start(20);
        assert_eq!(game.game_over(), None);
    }

    #[test]
    fn test_turn_handling_loss() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..4 {
            game.handle_board_move(x, 0, 2).unwrap();
        }

        let response = game.handle_turn(4, 0);

        assert!(!response.contains("ERROR"));
        assert!(!game.game_in_progress);
    }

    #[test]
    fn test_make_move_win() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 6..10 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }

        let response = game.handle_turn(0, 0);

        let parts: Vec<&str> = response.split(',').collect();
        let bot_x: usize = parts[0].parse().unwrap();
        let bot_y: usize = parts[1].parse().unwrap();

        assert_eq!(game.board.get_cell(bot_x, bot_y), Some(Cell::MyStone));
        assert_eq!(game.game_over(), Some(Cell::MyStone));
        assert!(!game.game_in_progress);
    }

    #[test]
    fn test_block_opponent_immediate_win() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..3 {
            game.board.set_cell(x, 5, Cell::OpStone).unwrap();
        }

        let response = game.handle_turn(3, 5);

        assert_eq!(response, "4,5");
        assert_eq!(game.board.get_cell(4, 5), Some(Cell::MyStone));
        assert_eq!(game.game_over(), None);
        assert!(game.game_in_progress);
    }
}

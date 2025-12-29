#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    MyStone,
    OpStone,
    Forbidden,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

pub struct GameState {
    size: usize,
    is_initialized: bool,
    game_in_progress: bool,
    board: [[Cell; 20]; 20],
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            size: 0,
            is_initialized: false,
            game_in_progress: false,
            board: [[Cell::default(); 20]; 20],
        }
    }

    pub fn handle_start(&mut self, size: usize) -> String {
        if size != 20 {
            return format!("ERROR unsupported size {}", size);
        }
        self.size = size;
        self.is_initialized = true;
        self.game_in_progress = false;
        self.board = [[Cell::Empty; 20]; 20];
        "OK".to_string()
    }

    pub fn handle_turn(&mut self, x: usize, y: usize) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        if x >= self.size || y >= self.size {
            return "ERROR coordinates out of range".to_string();
        }
        if self.board[x][y] != Cell::Empty {
            return "ERROR cell already occupied".to_string();
        }

        self.board[x][y] = Cell::OpStone;
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
        self.board = [[Cell::Empty; 20]; 20];
        true
    }

    pub fn handle_board_move(&mut self, x: usize, y: usize, field: usize) {
        if x < self.size && y < self.size {
            self.board[x][y] = match field {
                1 => Cell::MyStone,
                2 => Cell::OpStone,
                3 => Cell::Forbidden,
                _ => Cell::Empty,
            };
        }
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
        self.board = [[Cell::Empty; 20]; 20];
        "OK".to_string()
    }

    fn make_move(&mut self) -> String {
        // TODO: implement actual AI logic
        if self.board[10][10] == Cell::Empty {
            self.board[10][10] = Cell::MyStone;
            return "10,10".to_string();
        }

        for x in 0..self.size {
            for y in 0..self.size {
                if self.board[x][y] == Cell::Empty {
                    self.board[x][y] = Cell::MyStone;
                    return format!("{},{}", x, y);
                }
            }
        }
        "ERROR board full".to_string()
    }
}

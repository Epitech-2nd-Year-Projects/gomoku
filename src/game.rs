use crate::protocol::{Command, Response};

pub struct GameState {
    size: usize,
    board: Vec<u8>,
    is_initialized: bool,
    game_in_progress: bool,
    receiving_board_data: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            size: 20,
            board: vec![0; 20 * 20],
            is_initialized: false,
            game_in_progress: false,
            receiving_board_data: false,
        }
    }

    pub fn is_receiving_board_data(&self) -> bool {
        self.receiving_board_data
    }

    pub fn handle_board_data(&mut self, line: &str) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            if let (Ok(x), Ok(y), Ok(player)) = (
                parts[0].parse::<usize>(),
                parts[1].parse::<usize>(),
                parts[2].parse::<u8>(),
            ) {
                if x < self.size && y < self.size {
                    self.board[y * self.size + x] = player;
                }
            }
        }
    }

    pub fn handle_command(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::Start(size) => {
                if size < 5 {
                    return Response::Error("Board size too small".to_string());
                }
                self.size = size;
                self.board = vec![0; size * size];
                self.is_initialized = true;
                self.game_in_progress = false;
                self.receiving_board_data = false;
                Response::Ok
            }
            Command::Begin => {
                if !self.is_initialized {
                    return Response::Error("Game not initialized".to_string());
                }
                self.game_in_progress = true;
                let center = self.size / 2;
                self.board[center * self.size + center] = 1;
                Response::Move(center, center)
            }
            Command::Turn(x, y) => {
                if !self.is_initialized {
                    return Response::Error("Game not initialized".to_string());
                }
                if x >= self.size || y >= self.size {
                    return Response::Error("Invalid coordinates".to_string());
                }

                self.game_in_progress = true;
                self.board[y * self.size + x] = 2;

                let my_x = if x + 1 < self.size { x + 1 } else { 0 };
                let my_y = y;
                self.board[my_y * self.size + my_x] = 1;

                Response::Move(my_x, my_y)
            }
            Command::Board => {
                if !self.is_initialized {
                    return Response::Error("Game not initialized".to_string());
                }
                self.receiving_board_data = true;
                self.game_in_progress = true;
                self.board.fill(0);
                Response::None
            }
            Command::Done => {
                if self.receiving_board_data {
                    self.receiving_board_data = false;
                    // Placeholder move
                    let center = self.size / 2;
                    self.board[center * self.size + center] = 1;
                    return Response::Move(center, center);
                }
                Response::Unknown("Unexpected DONE".to_string())
            }
            Command::Info(_, _) => Response::None,
            Command::End => {
                self.game_in_progress = false;
                Response::None
            }
            Command::About => Response::Raw(
                "name=\"pbrain-brainrot\", version=\"0.1\", author=\"Brainrot\", country=\"FR\""
                    .to_string(),
            ),
            Command::Unknown => {
                if self.receiving_board_data {
                    Response::None
                } else {
                    Response::Unknown("Unknown command".to_string())
                }
            }
        }
    }
}

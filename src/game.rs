use crate::board::{Board, Cell};

const CANDIDATE_RADIUS: isize = 2;
const CANDIDATE_CAP: usize = 80;
const CENTER_CELLS: [(usize, usize); 4] = [(10, 10), (9, 9), (9, 10), (10, 9)];

const SCORE_OPEN_FOUR: i32 = 10000;
const SCORE_CLOSED_FOUR: i32 = 1000;
const SCORE_OPEN_THREE: i32 = 500;
const SCORE_CLOSED_THREE: i32 = 100;
const SCORE_OPEN_TWO: i32 = 10;
const SCORE_CLOSED_TWO: i32 = 1;

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

    fn count_stones(&self) -> usize {
        let mut count = 0;
        for y in 0..self.size {
            for x in 0..self.size {
                match self.board.get_cell(x, y) {
                    Some(Cell::MyStone) | Some(Cell::OpStone) => count += 1,
                    _ => {}
                }
            }
        }
        count
    }

    fn best_chain_len(&self, x: usize, y: usize, player: Cell) -> usize {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut best = 1;

        for (dx, dy) in directions {
            let mut count = 1;
            count += self.count_in_direction(x, y, dx, dy, player);
            count += self.count_in_direction(x, y, -dx, -dy, player);
            if count > best {
                best = count;
            }
        }

        best
    }

    fn count_in_direction(&self, x: usize, y: usize, dx: isize, dy: isize, player: Cell) -> usize {
        let mut count = 0;
        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;

        while nx >= 0 && ny >= 0 && nx < self.size as isize && ny < self.size as isize {
            if self.board.get_cell(nx as usize, ny as usize) == Some(player) {
                count += 1;
                nx += dx;
                ny += dy;
            } else {
                break;
            }
        }

        count
    }

    fn center_distance(&self, x: usize, y: usize) -> usize {
        let center = self.size / 2;
        let dx = if x > center { x - center } else { center - x };
        let dy = if y > center { y - center } else { center - y };
        dx.max(dy)
    }

    fn add_candidate(mask: &mut [bool; 400], size: usize, x: usize, y: usize) {
        let idx = y * size + x;
        mask[idx] = true;
    }

    fn generate_candidates(&self) -> Vec<(usize, usize)> {
        let total_stones = self.count_stones();
        if total_stones == 0 {
            let mut centers = Vec::new();
            for &(x, y) in &CENTER_CELLS {
                if self.board.is_empty(x, y) {
                    centers.push((x, y));
                }
            }
            if !centers.is_empty() {
                return centers;
            }
            return self.board.iter_empty().collect();
        }

        let early_game = total_stones <= 2;
        let mut mask = [false; 400];

        if early_game {
            for &(cx, cy) in &CENTER_CELLS {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = cx as isize + dx;
                        let ny = cy as isize + dy;
                        if nx < 0 || ny < 0 || nx >= self.size as isize || ny >= self.size as isize
                        {
                            continue;
                        }
                        let ux = nx as usize;
                        let uy = ny as usize;
                        if self.board.is_empty(ux, uy) {
                            GameState::add_candidate(&mut mask, self.size, ux, uy);
                        }
                    }
                }
            }
        }

        for y in 0..self.size {
            for x in 0..self.size {
                match self.board.get_cell(x, y) {
                    Some(Cell::MyStone) | Some(Cell::OpStone) => {
                        for dy in -CANDIDATE_RADIUS..=CANDIDATE_RADIUS {
                            for dx in -CANDIDATE_RADIUS..=CANDIDATE_RADIUS {
                                let nx = x as isize + dx;
                                let ny = y as isize + dy;
                                if nx < 0
                                    || ny < 0
                                    || nx >= self.size as isize
                                    || ny >= self.size as isize
                                {
                                    continue;
                                }
                                let ux = nx as usize;
                                let uy = ny as usize;
                                if self.board.is_empty(ux, uy) {
                                    GameState::add_candidate(&mut mask, self.size, ux, uy);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut candidates = Vec::new();
        for y in 0..self.size {
            for x in 0..self.size {
                if mask[y * self.size + x] {
                    candidates.push((x, y));
                }
            }
        }

        if candidates.is_empty() {
            candidates = self.board.iter_empty().collect();
        }

        let mut scored: Vec<(usize, usize, i32, usize)> = candidates
            .into_iter()
            .map(|(x, y)| {
                let my_best = self.best_chain_len(x, y, Cell::MyStone) as i32;
                let opp_best = self.best_chain_len(x, y, Cell::OpStone) as i32;
                let mut score = my_best * my_best * 10 + opp_best * opp_best * 12;
                let center_dist = self.center_distance(x, y);
                if early_game {
                    let bonus = (4_i32 - center_dist as i32).max(0);
                    score += bonus;
                }
                (x, y, score, center_dist)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| a.3.cmp(&b.3))
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.0.cmp(&b.0))
        });

        if scored.len() > CANDIDATE_CAP {
            scored.truncate(CANDIDATE_CAP);
        }

        scored.into_iter().map(|(x, y, _, _)| (x, y)).collect()
    }

    fn find_immediate_win(&mut self, player: Cell) -> Option<(usize, usize)> {
        let candidates = self.generate_candidates();

        for (x, y) in candidates {
            self.board
                .set_cell(x, y, player)
                .expect("board indices from candidates");
            let is_win = self.board.check_five_in_a_row(player);
            self.board
                .set_cell(x, y, Cell::Empty)
                .expect("board indices from candidates");

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
        for (x, y) in self.generate_candidates() {
            if self.validate_move(x, y).is_ok() {
                return Some((x, y));
            }
        }

        self.board
            .iter_empty()
            .find(|&(x, y)| self.validate_move(x, y).is_ok())
    }

    fn evaluate_sequence(&self, x: usize, y: usize, dx: isize, dy: isize, player: Cell) -> i32 {
        let mut forward_count = 0;
        let mut backward_count = 0;

        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;
        while nx >= 0
            && ny >= 0
            && nx < self.size as isize
            && ny < self.size as isize
        {
            if self.board.get_cell(nx as usize, ny as usize) == Some(player) {
                forward_count += 1;
                nx += dx;
                ny += dy;
            } else {
                break;
            }
        }

        let forward_open = nx >= 0
            && ny >= 0
            && nx < self.size as isize
            && ny < self.size as isize
            && self.board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

        nx = x as isize - dx;
        ny = y as isize - dy;
        while nx >= 0
            && ny >= 0
            && nx < self.size as isize
            && ny < self.size as isize
        {
            if self.board.get_cell(nx as usize, ny as usize) == Some(player) {
                backward_count += 1;
                nx -= dx;
                ny -= dy;
            } else {
                break;
            }
        }

        let backward_open = nx >= 0
            && ny >= 0
            && nx < self.size as isize
            && ny < self.size as isize
            && self.board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

        let total_count = forward_count + backward_count + 1;
        let open_sides = if forward_open { 1 } else { 0 } + if backward_open { 1 } else { 0 };

        if total_count >= 4 {
            if open_sides == 2 {
                SCORE_OPEN_FOUR
            } else {
                SCORE_CLOSED_FOUR
            }
        } else if total_count == 3 {
            if open_sides == 2 {
                SCORE_OPEN_THREE
            } else {
                SCORE_CLOSED_THREE
            }
        } else if total_count == 2 {
            if open_sides == 2 {
                SCORE_OPEN_TWO
            } else {
                SCORE_CLOSED_TWO
            }
        } else {
            0
        }
    }

    fn evaluate(&self, player: Cell) -> i32 {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut score = 0;
        let mut processed = [[[false; 20]; 20]; 4];

        for y in 0..self.size {
            for x in 0..self.size {
                if self.board.get_cell(x, y) != Some(player) {
                    continue;
                }

                for (dir_idx, &(dx, dy)) in directions.iter().enumerate() {
                    if processed[dir_idx][y][x] {
                        continue;
                    }

                    let pattern_score = self.evaluate_sequence(x, y, dx, dy, player);
                    if pattern_score > 0 {
                        score += pattern_score;

                        let mut nx = x as isize;
                        let mut ny = y as isize;
                        while nx >= 0
                            && ny >= 0
                            && nx < self.size as isize
                            && ny < self.size as isize
                        {
                            if self.board.get_cell(nx as usize, ny as usize) != Some(player) {
                                break;
                            }
                            processed[dir_idx][ny as usize][nx as usize] = true;
                            nx += dx;
                            ny += dy;
                        }

                        nx = x as isize - dx;
                        ny = y as isize - dy;
                        while nx >= 0
                            && ny >= 0
                            && nx < self.size as isize
                            && ny < self.size as isize
                        {
                            if self.board.get_cell(nx as usize, ny as usize) != Some(player) {
                                break;
                            }
                            processed[dir_idx][ny as usize][nx as usize] = true;
                            nx -= dx;
                            ny -= dy;
                        }
                    }
                }
            }
        }

        score
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
    fn test_fallback_move_center_then_nearby() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert_eq!(game.fallback_move(), Some((10, 10)));

        game.board.set_cell(10, 10, Cell::OpStone).unwrap();
        let response = game.fallback_move();
        assert!(response.is_some());
        assert_ne!(response, Some((10, 10)));
    }

    #[test]
    fn test_generate_candidates_empty_board_centers() {
        let mut game = GameState::new();
        game.handle_start(20);

        let candidates = game.generate_candidates();
        let expected = vec![(10, 10), (9, 9), (9, 10), (10, 9)];
        assert_eq!(candidates, expected);
    }

    #[test]
    fn test_generate_candidates_early_game_center_neighborhood() {
        let mut game = GameState::new();
        game.handle_start(20);
        game.board.set_cell(0, 0, Cell::MyStone).unwrap();

        let candidates = game.generate_candidates();
        assert!(candidates.contains(&(8, 8)));
        assert!(candidates.contains(&(11, 11)));
    }

    #[test]
    fn test_generate_candidates_excludes_forbidden_cells() {
        let mut game = GameState::new();
        game.handle_start(20);
        game.board.set_cell(6, 6, Cell::MyStone).unwrap();
        game.board.set_cell(5, 5, Cell::Forbidden).unwrap();

        let candidates = game.generate_candidates();
        assert!(!candidates.contains(&(5, 5)));
        assert!(candidates.contains(&(6, 7)));
    }

    #[test]
    fn test_generate_candidates_bounds_and_radius() {
        let mut game = GameState::new();
        game.handle_start(20);
        game.board.set_cell(0, 0, Cell::MyStone).unwrap();

        let candidates = game.generate_candidates();
        assert!(candidates.contains(&(2, 2)));
        assert!(!candidates.contains(&(3, 3)));
    }

    #[test]
    fn test_generate_candidates_cap() {
        let mut game = GameState::new();
        game.handle_start(20);

        for y in 0..20 {
            for x in 0..20 {
                if (x + y) % 3 == 0 {
                    game.board.set_cell(x, y, Cell::MyStone).unwrap();
                }
            }
        }

        let candidates = game.generate_candidates();
        assert_eq!(candidates.len(), 80);
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

    #[test]
    fn test_prioritize_win_over_block() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 6..10 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }
        game.board.set_cell(5, 10, Cell::OpStone).unwrap();

        for x in 0..4 {
            game.board.set_cell(x, 5, Cell::OpStone).unwrap();
        }

        let response = game.handle_turn(15, 15);

        assert_eq!(response, "10,10");
        assert_eq!(game.board.get_cell(10, 10), Some(Cell::MyStone));
        assert_eq!(game.game_over(), Some(Cell::MyStone));
        assert!(!game.game_in_progress);
    }

    #[test]
    fn test_evaluate_open_four() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 1..5 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }

        let score = game.evaluate(Cell::MyStone);
        assert!(score >= SCORE_OPEN_FOUR);
    }

    #[test]
    fn test_evaluate_closed_four() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(0, 10, Cell::OpStone).unwrap();
        for x in 1..5 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }

        let score = game.evaluate(Cell::MyStone);
        assert!(score >= SCORE_CLOSED_FOUR);
    }

    #[test]
    fn test_evaluate_open_three() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 1..4 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }

        let score = game.evaluate(Cell::MyStone);
        assert!(score >= SCORE_OPEN_THREE);
    }

    #[test]
    fn test_evaluate_empty_board() {
        let mut game = GameState::new();
        game.handle_start(20);

        let score = game.evaluate(Cell::MyStone);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_evaluate_opponent_patterns() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 1..5 {
            game.board.set_cell(x, 5, Cell::OpStone).unwrap();
        }

        let my_score = game.evaluate(Cell::MyStone);
        let opp_score = game.evaluate(Cell::OpStone);

        assert_eq!(my_score, 0);
        assert!(opp_score >= SCORE_OPEN_FOUR);
    }

    #[test]
    fn test_evaluate_multiple_patterns() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 1..4 {
            game.board.set_cell(x, 10, Cell::MyStone).unwrap();
        }

        for y in 1..4 {
            game.board.set_cell(10, y, Cell::MyStone).unwrap();
        }

        let score = game.evaluate(Cell::MyStone);
        assert!(score >= SCORE_OPEN_THREE * 2);
    }
}

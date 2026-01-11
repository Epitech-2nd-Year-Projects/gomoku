use crate::board::{Board, Cell};
use crate::zobrist::{TTFlag, TranspositionTable, ZobristKeys};
use std::time::{Duration, Instant};

const CANDIDATE_RADIUS: isize = 2;
const CANDIDATE_CAP: usize = 80;
const CENTER_CELLS: [(usize, usize); 4] = [(10, 10), (9, 9), (9, 10), (10, 9)];

const SCORE_OPEN_FOUR: i32 = 50000;
const SCORE_CLOSED_FOUR: i32 = 10000;
const SCORE_OPEN_THREE: i32 = 5000;
const SCORE_CLOSED_THREE: i32 = 500;
const SCORE_OPEN_TWO: i32 = 100;
const SCORE_CLOSED_TWO: i32 = 10;

const SCORE_DOUBLE_THREAT: i32 = 80000;

const TIME_BUDGET: Duration = Duration::from_secs(5);
const MAX_SEARCH_DEPTH: usize = 20;

const BOARD_SIZE: usize = 20;
const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

#[derive(Default, Clone, Copy)]
pub struct ThreatInfo {
    pub open_fours: u8,
    pub closed_fours: u8,
    pub open_threes: u8,
}

impl ThreatInfo {
    pub fn is_winning(&self) -> bool {
        self.open_fours >= 1
            || self.open_threes >= 2
            || self.closed_fours >= 2
            || (self.closed_fours >= 1 && self.open_threes >= 1)
    }

    pub fn score(&self) -> i32 {
        if self.is_winning() {
            return SCORE_DOUBLE_THREAT;
        }
        let mut s = 0i32;
        s += self.open_fours as i32 * SCORE_OPEN_FOUR;
        s += self.closed_fours as i32 * SCORE_CLOSED_FOUR;
        s += self.open_threes as i32 * SCORE_OPEN_THREE;
        s
    }
}

#[derive(Clone)]
pub struct IncrementalScores {
    scores: [[[i32; 4]; 400]; 2],
    totals: [i32; 2],
}

impl Default for IncrementalScores {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalScores {
    pub fn new() -> Self {
        Self {
            scores: [[[0; 4]; 400]; 2],
            totals: [0; 2],
        }
    }

    pub fn clear(&mut self) {
        self.scores = [[[0; 4]; 400]; 2];
        self.totals = [0; 2];
    }

    #[inline]
    fn player_index(player: Cell) -> usize {
        match player {
            Cell::MyStone => 0,
            Cell::OpStone => 1,
            _ => 0,
        }
    }

    #[inline]
    fn cell_index(x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }

    pub fn evaluate_position(&self) -> i32 {
        self.totals[0] - self.totals[1]
    }

    fn evaluate_sequence_for_cell(
        board: &Board,
        x: usize,
        y: usize,
        dir_idx: usize,
        player: Cell,
    ) -> i32 {
        let (dx, dy) = DIRECTIONS[dir_idx];
        let size = BOARD_SIZE as isize;

        let mut forward_count = 0;
        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;
        while nx >= 0 && ny >= 0 && nx < size && ny < size {
            if board.get_cell(nx as usize, ny as usize) == Some(player) {
                forward_count += 1;
                nx += dx;
                ny += dy;
            } else {
                break;
            }
        }

        let forward_open = nx >= 0
            && ny >= 0
            && nx < size
            && ny < size
            && board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

        let mut backward_count = 0;
        nx = x as isize - dx;
        ny = y as isize - dy;
        while nx >= 0 && ny >= 0 && nx < size && ny < size {
            if board.get_cell(nx as usize, ny as usize) == Some(player) {
                backward_count += 1;
                nx -= dx;
                ny -= dy;
            } else {
                break;
            }
        }

        let backward_open = nx >= 0
            && ny >= 0
            && nx < size
            && ny < size
            && board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

        let total_count = forward_count + backward_count + 1;
        let open_sides = i32::from(forward_open) + i32::from(backward_open);

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

    fn collect_affected_stones(
        board: &Board,
        x: usize,
        y: usize,
        dir_idx: usize,
    ) -> Vec<(usize, usize, Cell)> {
        let (dx, dy) = DIRECTIONS[dir_idx];
        let size = BOARD_SIZE as isize;
        let mut affected = Vec::new();

        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;
        while nx >= 0 && ny >= 0 && nx < size && ny < size {
            match board.get_cell(nx as usize, ny as usize) {
                Some(Cell::MyStone) => {
                    affected.push((nx as usize, ny as usize, Cell::MyStone));
                    nx += dx;
                    ny += dy;
                }
                Some(Cell::OpStone) => {
                    affected.push((nx as usize, ny as usize, Cell::OpStone));
                    nx += dx;
                    ny += dy;
                }
                _ => break,
            }
        }

        nx = x as isize - dx;
        ny = y as isize - dy;
        while nx >= 0 && ny >= 0 && nx < size && ny < size {
            match board.get_cell(nx as usize, ny as usize) {
                Some(Cell::MyStone) => {
                    affected.push((nx as usize, ny as usize, Cell::MyStone));
                    nx -= dx;
                    ny -= dy;
                }
                Some(Cell::OpStone) => {
                    affected.push((nx as usize, ny as usize, Cell::OpStone));
                    nx -= dx;
                    ny -= dy;
                }
                _ => break,
            }
        }

        affected
    }

    fn update_cell_score(
        &mut self,
        board: &Board,
        x: usize,
        y: usize,
        dir_idx: usize,
        player: Cell,
    ) {
        let idx = Self::cell_index(x, y);
        let p_idx = Self::player_index(player);

        let old_score = self.scores[p_idx][idx][dir_idx];
        self.totals[p_idx] -= old_score;

        let new_score = Self::evaluate_sequence_for_cell(board, x, y, dir_idx, player);
        self.scores[p_idx][idx][dir_idx] = new_score;
        self.totals[p_idx] += new_score;
    }

    fn clear_cell_scores(&mut self, x: usize, y: usize, player: Cell) {
        let idx = Self::cell_index(x, y);
        let p_idx = Self::player_index(player);

        for dir_idx in 0..4 {
            let old_score = self.scores[p_idx][idx][dir_idx];
            self.totals[p_idx] -= old_score;
            self.scores[p_idx][idx][dir_idx] = 0;
        }
    }

    pub fn on_stone_placed(&mut self, board: &Board, x: usize, y: usize, player: Cell) {
        for dir_idx in 0..4 {
            let affected = Self::collect_affected_stones(board, x, y, dir_idx);
            for (ax, ay, ap) in &affected {
                self.update_cell_score(board, *ax, *ay, dir_idx, *ap);
            }
            self.update_cell_score(board, x, y, dir_idx, player);
        }
    }

    pub fn on_stone_removed(&mut self, board: &Board, x: usize, y: usize, old_player: Cell) {
        self.clear_cell_scores(x, y, old_player);

        for dir_idx in 0..4 {
            let affected = Self::collect_affected_stones(board, x, y, dir_idx);
            for (ax, ay, ap) in &affected {
                self.update_cell_score(board, *ax, *ay, dir_idx, *ap);
            }
        }
    }

    pub fn rebuild_from_board(&mut self, board: &Board) {
        self.clear();

        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let cell = board.get_cell(x, y);
                if cell == Some(Cell::MyStone) || cell == Some(Cell::OpStone) {
                    let player = cell.unwrap();
                    let p_idx = Self::player_index(player);
                    let idx = Self::cell_index(x, y);

                    for dir_idx in 0..4 {
                        let score = Self::evaluate_sequence_for_cell(board, x, y, dir_idx, player);
                        self.scores[p_idx][idx][dir_idx] = score;
                        self.totals[p_idx] += score;
                    }
                }
            }
        }
    }
}

pub struct GameState {
    size: usize,
    is_initialized: bool,
    game_in_progress: bool,
    board: Board,
    zobrist: ZobristKeys,
    tt: TranspositionTable,
    inc_scores: IncrementalScores,
    killer_moves: [[Option<(usize, usize)>; 2]; MAX_SEARCH_DEPTH],
    history: [[i32; 400]; 2],
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            size: 0,
            is_initialized: false,
            game_in_progress: false,
            board: Board::default(),
            zobrist: ZobristKeys::new(),
            tt: TranspositionTable::new(),
            inc_scores: IncrementalScores::new(),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            history: [[0; 400]; 2],
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
        self.tt.clear();
        self.inc_scores.clear();
        self.killer_moves = [[None; 2]; MAX_SEARCH_DEPTH];
        self.history = [[0; 400]; 2];
        "OK".to_string()
    }

    #[inline]
    fn place_stone(&mut self, x: usize, y: usize, cell: Cell) {
        if let Some(idx) = self.board.get_index(x, y) {
            let old_cell = self.board.get_cell(x, y).unwrap_or(Cell::Empty);
            if old_cell != Cell::Empty {
                self.board
                    .update_hash(self.zobrist.stone_key(idx, old_cell));
            }
            self.board.set_cell(x, y, cell).unwrap();
            if cell != Cell::Empty {
                self.board.update_hash(self.zobrist.stone_key(idx, cell));
                self.inc_scores.on_stone_placed(&self.board, x, y, cell);
            }
        }
    }

    #[inline]
    fn remove_stone(&mut self, x: usize, y: usize) {
        if let Some(idx) = self.board.get_index(x, y) {
            let old_cell = self.board.get_cell(x, y).unwrap_or(Cell::Empty);
            if old_cell != Cell::Empty {
                self.board
                    .update_hash(self.zobrist.stone_key(idx, old_cell));
            }
            self.board.set_cell(x, y, Cell::Empty).unwrap();
            if old_cell == Cell::MyStone || old_cell == Cell::OpStone {
                self.inc_scores
                    .on_stone_removed(&self.board, x, y, old_cell);
            }
        }
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
        if !self.is_initialized {
            self.handle_start(20);
        }

        if let Err(e) = self.validate_move(x, y) {
            return e.to_string();
        }

        self.place_stone(x, y, Cell::OpStone);
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
            self.handle_start(20);
        }
        self.game_in_progress = true;
        self.generate_move()
    }

    pub fn handle_board_start(&mut self) -> Result<(), &'static str> {
        if !self.is_initialized {
            self.handle_start(20);
        }
        self.game_in_progress = true;
        self.board.clear();
        self.inc_scores.clear();
        Ok(())
    }

    pub fn handle_board_move(
        &mut self,
        x: usize,
        y: usize,
        field: usize,
    ) -> Result<(), &'static str> {
        if !self.is_initialized {
            self.handle_start(20);
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

        self.place_stone(x, y, cell);
        Ok(())
    }

    pub fn handle_board_done(&mut self) -> String {
        if !self.is_initialized {
            self.handle_start(20);
        }
        self.inc_scores.rebuild_from_board(&self.board);
        self.generate_move()
    }

    pub fn handle_restart(&mut self) -> String {
        if !self.is_initialized {
            return "ERROR game not initialized".to_string();
        }
        self.game_in_progress = false;
        self.board.clear();
        self.tt.clear();
        self.inc_scores.clear();
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
                let my_threats = self.detect_threats(x, y, Cell::MyStone);
                let opp_threats = self.detect_threats(x, y, Cell::OpStone);

                let mut score = my_threats.score();
                if opp_threats.is_winning() {
                    score = score.max(SCORE_DOUBLE_THREAT - 1000);
                } else {
                    score += opp_threats.score() / 2;
                }

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
            self.place_stone(x, y, player);
            let is_win = self.board.check_five_in_a_row(player);
            self.remove_stone(x, y);

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
            .or_else(|| self.find_best_move())
            .or_else(|| self.fallback_move())
            .or_else(|| self.any_empty_cell());

        if let Some((x, y)) = move_coords {
            self.place_stone(x, y, Cell::MyStone);

            if self.game_over().is_some() {
                self.game_in_progress = false;
            }

            return format!("{},{}", x, y);
        }

        self.emergency_move()
    }

    fn any_empty_cell(&self) -> Option<(usize, usize)> {
        self.board.iter_empty().next()
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

    pub fn emergency_move(&self) -> String {
        if let Some((x, y)) = self.board.iter_empty().next() {
            return format!("{},{}", x, y);
        }
        for y in 0..20 {
            for x in 0..20 {
                if self.board.get_cell(x, y) == Some(Cell::Empty) {
                    return format!("{},{}", x, y);
                }
            }
        }
        "10,10".to_string()
    }

    fn generate_forcing_moves(&self, player: Cell) -> Vec<(usize, usize)> {
        let candidates = self.generate_candidates();
        let mut forcing = Vec::new();

        for (x, y) in candidates {
            let my_threats = self.detect_threats(x, y, player);
            let opp = if player == Cell::MyStone {
                Cell::OpStone
            } else {
                Cell::MyStone
            };
            let opp_threats = self.detect_threats(x, y, opp);

            if my_threats.open_fours >= 1
                || my_threats.closed_fours >= 1
                || my_threats.open_threes >= 2
                || opp_threats.open_fours >= 1
                || opp_threats.closed_fours >= 1
            {
                forcing.push((x, y));
            }
        }
        forcing
    }

    fn quiescence(
        &mut self,
        mut alpha: i32,
        beta: i32,
        player: Cell,
        deadline: Instant,
        qdepth: usize,
    ) -> Option<i32> {
        if Instant::now() >= deadline {
            return None;
        }

        if let Some(winner) = self.game_over() {
            match winner {
                Cell::MyStone => {
                    return Some(if player == Cell::MyStone {
                        100000
                    } else {
                        -100000
                    });
                }
                Cell::OpStone => {
                    return Some(if player == Cell::OpStone {
                        100000
                    } else {
                        -100000
                    });
                }
                Cell::Empty => return Some(0),
                _ => {}
            }
        }

        let stand_pat = self.evaluate_position();
        let stand_pat = if player == Cell::MyStone {
            stand_pat
        } else {
            -stand_pat
        };

        if stand_pat >= beta {
            return Some(beta);
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if qdepth == 0 {
            return Some(alpha);
        }

        let forcing = self.generate_forcing_moves(player);
        if forcing.is_empty() {
            return Some(alpha);
        }

        for (x, y) in forcing {
            if self.validate_move(x, y).is_err() {
                continue;
            }

            self.place_stone(x, y, player);
            let next_player = if player == Cell::MyStone {
                Cell::OpStone
            } else {
                Cell::MyStone
            };
            let score = -self.quiescence(-beta, -alpha, next_player, deadline, qdepth - 1)?;
            self.remove_stone(x, y);

            if score >= beta {
                return Some(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }

        Some(alpha)
    }

    fn negamax(
        &mut self,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        player: Cell,
        deadline: Instant,
    ) -> Option<i32> {
        if Instant::now() >= deadline {
            return None;
        }

        if let Some(winner) = self.game_over() {
            match winner {
                Cell::MyStone => {
                    return Some(if player == Cell::MyStone {
                        100000
                    } else {
                        -100000
                    });
                }
                Cell::OpStone => {
                    return Some(if player == Cell::OpStone {
                        100000
                    } else {
                        -100000
                    });
                }
                Cell::Empty => return Some(0),
                _ => {}
            }
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, player, deadline, 4);
        }

        let hash = self.compute_hash_with_turn(player);
        let original_alpha = alpha;

        if let Some(entry) = self.tt.probe(hash) {
            if entry.depth as usize >= depth {
                match entry.flag {
                    TTFlag::Exact => return Some(entry.score),
                    TTFlag::LowerBound => {
                        if entry.score > alpha {
                            alpha = entry.score;
                        }
                    }
                    TTFlag::UpperBound => {
                        if entry.score < beta {
                            return Some(entry.score);
                        }
                    }
                }
                if alpha >= beta {
                    return Some(entry.score);
                }
            }
        }

        let mut candidates = self.generate_candidates();
        if candidates.is_empty() {
            let eval = self.evaluate_position();
            return Some(if player == Cell::MyStone { eval } else { -eval });
        }

        let tt_move = self.tt.probe(hash).and_then(|e| e.best_move);
        let player_idx = if player == Cell::MyStone { 0 } else { 1 };

        candidates.sort_by(|&(ax, ay), &(bx, by)| {
            let a_is_tt = tt_move == Some((ax as u8, ay as u8));
            let b_is_tt = tt_move == Some((bx as u8, by as u8));
            if a_is_tt != b_is_tt {
                return b_is_tt.cmp(&a_is_tt);
            }

            let a_is_killer = depth < MAX_SEARCH_DEPTH
                && (self.killer_moves[depth][0] == Some((ax, ay))
                    || self.killer_moves[depth][1] == Some((ax, ay)));
            let b_is_killer = depth < MAX_SEARCH_DEPTH
                && (self.killer_moves[depth][0] == Some((bx, by))
                    || self.killer_moves[depth][1] == Some((bx, by)));
            if a_is_killer != b_is_killer {
                return b_is_killer.cmp(&a_is_killer);
            }

            let a_hist = self.history[player_idx][ay * 20 + ax];
            let b_hist = self.history[player_idx][by * 20 + bx];
            b_hist.cmp(&a_hist)
        });

        let mut best_value = -200000;
        let mut best_move = None;
        for (x, y) in candidates {
            if self.validate_move(x, y).is_err() {
                continue;
            }

            self.place_stone(x, y, player);
            let next_player = if player == Cell::MyStone {
                Cell::OpStone
            } else {
                Cell::MyStone
            };
            let value = -self.negamax(depth - 1, -beta, -alpha, next_player, deadline)?;
            self.remove_stone(x, y);

            if value > best_value {
                best_value = value;
                best_move = Some((x, y));
            }
            if value > alpha {
                alpha = value;
            }
            if alpha >= beta {
                if depth < MAX_SEARCH_DEPTH {
                    if self.killer_moves[depth][0] != Some((x, y)) {
                        self.killer_moves[depth][1] = self.killer_moves[depth][0];
                        self.killer_moves[depth][0] = Some((x, y));
                    }
                    let player_idx = if player == Cell::MyStone { 0 } else { 1 };
                    let idx = y * 20 + x;
                    self.history[player_idx][idx] += (depth * depth) as i32;
                }
                break;
            }
        }

        let flag = if best_value <= original_alpha {
            TTFlag::UpperBound
        } else if best_value >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        self.tt
            .store(hash, depth as u8, best_value, flag, best_move);

        Some(best_value)
    }

    #[inline]
    fn compute_hash_with_turn(&self, player: Cell) -> u64 {
        let mut hash = self.board.hash();
        if player == Cell::OpStone {
            hash ^= self.zobrist.turn_key();
        }
        hash
    }

    fn find_best_move(&mut self) -> Option<(usize, usize)> {
        let mut candidates = self.generate_candidates();
        if candidates.is_empty() {
            return None;
        }

        let hash = self.compute_hash_with_turn(Cell::MyStone);
        if let Some(entry) = self.tt.probe(hash) {
            if let Some((tx, ty)) = entry.best_move {
                let tx = tx as usize;
                let ty = ty as usize;
                if let Some(pos) = candidates.iter().position(|&(x, y)| x == tx && y == ty) {
                    candidates.swap(0, pos);
                }
            }
        }

        let deadline = Instant::now() + TIME_BUDGET;
        let mut best_move: Option<(usize, usize)> = None;

        for depth in 1..=MAX_SEARCH_DEPTH {
            if Instant::now() >= deadline {
                break;
            }

            let mut depth_best_move = None;
            let mut alpha = -200000;
            let beta = 200000;
            let mut search_completed = true;

            for (x, y) in &candidates {
                if self.validate_move(*x, *y).is_err() {
                    continue;
                }

                self.place_stone(*x, *y, Cell::MyStone);
                let result = self.negamax(depth - 1, -beta, -alpha, Cell::OpStone, deadline);
                self.remove_stone(*x, *y);

                match result {
                    Some(child_value) => {
                        let value = -child_value;
                        if value > alpha {
                            alpha = value;
                            depth_best_move = Some((*x, *y));
                        }
                    }
                    None => {
                        search_completed = false;
                        break;
                    }
                }
            }

            if search_completed {
                best_move = depth_best_move;
                if let Some((bx, by)) = depth_best_move {
                    if let Some(pos) = candidates.iter().position(|&(x, y)| x == bx && y == by) {
                        candidates.swap(0, pos);
                    }
                }
            }
        }

        best_move
    }

    #[cfg(test)]
    fn evaluate(&self, player: Cell) -> i32 {
        let mut total_score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for y in 0..self.size {
            for x in 0..self.size {
                if self.board.get_cell(x, y) != Some(player) {
                    continue;
                }

                for &(dx, dy) in &directions {
                    total_score += self.evaluate_sequence(x, y, dx, dy, player);
                }
            }
        }

        total_score
    }

    fn evaluate_position(&self) -> i32 {
        self.inc_scores.evaluate_position()
    }

    #[cfg(test)]
    fn evaluate_position_full_scan(&self) -> i32 {
        let my_score = self.evaluate(Cell::MyStone);
        let opp_score = self.evaluate(Cell::OpStone);
        my_score - opp_score
    }

    #[cfg(test)]
    fn evaluate_sequence(&self, x: usize, y: usize, dx: isize, dy: isize, player: Cell) -> i32 {
        let mut forward_count = 0;
        let mut backward_count = 0;

        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;
        while nx >= 0 && ny >= 0 && nx < self.size as isize && ny < self.size as isize {
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
        while nx >= 0 && ny >= 0 && nx < self.size as isize && ny < self.size as isize {
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

    fn detect_threats(&self, x: usize, y: usize, player: Cell) -> ThreatInfo {
        let mut info = ThreatInfo::default();
        let size = self.size as isize;

        for &(dx, dy) in &DIRECTIONS {
            let mut forward_count = 0i32;
            let mut nx = x as isize + dx;
            let mut ny = y as isize + dy;
            while nx >= 0 && ny >= 0 && nx < size && ny < size {
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
                && nx < size
                && ny < size
                && self.board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

            let mut backward_count = 0i32;
            nx = x as isize - dx;
            ny = y as isize - dy;
            while nx >= 0 && ny >= 0 && nx < size && ny < size {
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
                && nx < size
                && ny < size
                && self.board.get_cell(nx as usize, ny as usize) == Some(Cell::Empty);

            let total = forward_count + backward_count + 1;
            let open_sides = u8::from(forward_open) + u8::from(backward_open);

            if total >= 5 {
                info.open_fours += 1;
            } else if total == 4 {
                if open_sides == 2 {
                    info.open_fours += 1;
                } else if open_sides >= 1 {
                    info.closed_fours += 1;
                }
            } else if total == 3 && open_sides == 2 {
                info.open_threes += 1;
            }
        }
        info
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
    fn test_auto_initialize_on_commands() {
        let mut game = GameState::new();
        assert!(!game.is_initialized);

        assert!(game.handle_board_start().is_ok());
        assert!(game.is_initialized);

        let mut game2 = GameState::new();
        let response = game2.handle_begin();
        assert!(!response.contains("ERROR"));
        assert!(game2.is_initialized);

        let mut game3 = GameState::new();
        let response = game3.handle_turn(5, 5);
        assert!(!response.contains("ERROR"));
        assert!(game3.is_initialized);
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
            game.place_stone(x, 10, Cell::MyStone);
        }
        game.place_stone(5, 10, Cell::OpStone);

        for x in 0..4 {
            game.place_stone(x, 5, Cell::OpStone);
        }

        let response = game.handle_turn(15, 15);

        assert_eq!(response, "10,10");
        assert_eq!(game.board.get_cell(10, 10), Some(Cell::MyStone));
        assert_eq!(game.game_over(), Some(Cell::MyStone));
        assert!(!game.game_in_progress);
    }

    #[test]
    fn test_emergency_move_returns_valid_coords() {
        let game = GameState::new();
        let response = game.emergency_move();
        let parts: Vec<&str> = response.split(',').collect();
        assert_eq!(parts.len(), 2);
        let x: usize = parts[0].parse().unwrap();
        let y: usize = parts[1].parse().unwrap();
        assert!(x < 20 && y < 20);
    }

    #[test]
    fn test_emergency_move_full_board() {
        let mut game = GameState::new();
        game.handle_start(20);
        for y in 0..20 {
            for x in 0..20 {
                game.board.set_cell(x, y, Cell::MyStone).unwrap();
            }
        }
        let response = game.emergency_move();
        assert_eq!(response, "10,10");
    }

    #[test]
    fn test_generate_move_never_errors() {
        let mut game = GameState::new();
        game.handle_start(20);
        for y in 0..20 {
            for x in 0..20 {
                game.board.set_cell(x, y, Cell::MyStone).unwrap();
            }
        }
        let response = game.generate_move();
        assert!(!response.starts_with("ERROR"));
    }

    #[test]
    fn test_out_of_bounds_coordinates() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert!(game.handle_turn(100, 100).contains("ERROR"));
        assert!(game.handle_turn(0, 100).contains("ERROR"));
        assert!(game.handle_turn(100, 0).contains("ERROR"));
    }

    #[test]
    fn test_incremental_scores_single_stone() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.place_stone(10, 10, Cell::MyStone);

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_incremental_scores_horizontal_line() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 5..9 {
            game.place_stone(x, 10, Cell::MyStone);
        }

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_incremental_scores_mixed_stones() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.place_stone(10, 10, Cell::MyStone);
        game.place_stone(11, 10, Cell::MyStone);
        game.place_stone(12, 10, Cell::OpStone);
        game.place_stone(10, 11, Cell::OpStone);
        game.place_stone(10, 12, Cell::OpStone);

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_incremental_scores_after_removal() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.place_stone(10, 10, Cell::MyStone);
        game.place_stone(11, 10, Cell::MyStone);
        game.place_stone(12, 10, Cell::MyStone);

        let before_removal = game.evaluate_position();
        assert_eq!(before_removal, game.evaluate_position_full_scan());

        game.remove_stone(11, 10);

        let after_removal = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(after_removal, full_scan);
    }

    #[test]
    fn test_incremental_scores_place_remove_place() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.place_stone(10, 10, Cell::MyStone);
        game.place_stone(11, 10, Cell::OpStone);

        game.remove_stone(11, 10);
        game.place_stone(11, 10, Cell::MyStone);

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_incremental_scores_diagonal() {
        let mut game = GameState::new();
        game.handle_start(20);

        for i in 0..4 {
            game.place_stone(5 + i, 5 + i, Cell::MyStone);
        }
        game.place_stone(4, 4, Cell::OpStone);

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_incremental_scores_complex_pattern() {
        let mut game = GameState::new();
        game.handle_start(20);

        let moves = [
            (10, 10, Cell::MyStone),
            (11, 10, Cell::OpStone),
            (9, 10, Cell::MyStone),
            (10, 9, Cell::OpStone),
            (10, 11, Cell::MyStone),
            (12, 10, Cell::OpStone),
            (8, 10, Cell::MyStone),
            (10, 8, Cell::OpStone),
        ];

        for (x, y, player) in moves {
            game.place_stone(x, y, player);
            assert_eq!(
                game.evaluate_position(),
                game.evaluate_position_full_scan(),
                "Mismatch after placing stone at ({}, {})",
                x,
                y
            );
        }
    }

    #[test]
    fn test_incremental_scores_many_removals() {
        let mut game = GameState::new();
        game.handle_start(20);

        for i in 0..5 {
            game.place_stone(5 + i, 10, Cell::MyStone);
        }
        assert_eq!(game.evaluate_position(), game.evaluate_position_full_scan());

        for i in (0..5).rev() {
            game.remove_stone(5 + i, 10);
            assert_eq!(
                game.evaluate_position(),
                game.evaluate_position_full_scan(),
                "Mismatch after removing stone at ({}, 10)",
                5 + i
            );
        }

        assert_eq!(game.evaluate_position(), 0);
    }

    #[test]
    fn test_incremental_scores_rebuild() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(10, 10, Cell::MyStone).unwrap();
        game.board.set_cell(11, 10, Cell::MyStone).unwrap();
        game.board.set_cell(12, 10, Cell::OpStone).unwrap();

        game.inc_scores.rebuild_from_board(&game.board);

        let incremental = game.evaluate_position();
        let full_scan = game.evaluate_position_full_scan();
        assert_eq!(incremental, full_scan);
    }

    #[test]
    fn test_validate_move_all_corners() {
        let mut game = GameState::new();
        game.handle_start(20);

        let corners = [(0, 0), (19, 0), (0, 19), (19, 19)];
        for &(x, y) in &corners {
            assert!(
                game.validate_move(x, y).is_ok(),
                "Corner ({}, {}) should be valid",
                x,
                y
            );
        }
    }

    #[test]
    fn test_validate_move_edge_cells() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..20 {
            assert!(
                game.validate_move(x, 0).is_ok(),
                "Edge cell ({}, 0) should be valid",
                x
            );
            assert!(
                game.validate_move(x, 19).is_ok(),
                "Edge cell ({}, 19) should be valid",
                x
            );
        }
        for y in 1..19 {
            assert!(
                game.validate_move(0, y).is_ok(),
                "Edge cell (0, {}) should be valid",
                y
            );
            assert!(
                game.validate_move(19, y).is_ok(),
                "Edge cell (19, {}) should be valid",
                y
            );
        }
    }

    #[test]
    fn test_validate_move_occupied_by_my_stone() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(5, 5, Cell::MyStone).unwrap();
        assert_eq!(game.validate_move(5, 5), Err("ERROR cell already occupied"));
    }

    #[test]
    fn test_validate_move_occupied_by_opponent() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(5, 5, Cell::OpStone).unwrap();
        assert_eq!(game.validate_move(5, 5), Err("ERROR cell already occupied"));
    }

    #[test]
    fn test_validate_move_forbidden_cell() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(7, 7, Cell::Forbidden).unwrap();
        assert_eq!(game.validate_move(7, 7), Err("ERROR move forbidden"));
    }

    #[test]
    fn test_validate_move_out_of_bounds_boundary() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert!(game.validate_move(19, 19).is_ok());

        assert_eq!(
            game.validate_move(20, 0),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.validate_move(0, 20),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.validate_move(20, 19),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.validate_move(19, 20),
            Err("ERROR coordinates out of range")
        );
    }

    #[test]
    fn test_validate_move_large_out_of_bounds() {
        let mut game = GameState::new();
        game.handle_start(20);

        assert_eq!(
            game.validate_move(100, 0),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.validate_move(0, 100),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.validate_move(1000, 1000),
            Err("ERROR coordinates out of range")
        );
    }

    #[test]
    fn test_validate_move_not_initialized() {
        let game = GameState::new();
        assert_eq!(
            game.validate_move(10, 10),
            Err("ERROR game not initialized")
        );
    }

    #[test]
    fn test_turn_at_corners() {
        let mut game = GameState::new();
        game.handle_start(20);

        let response = game.handle_turn(0, 0);
        assert!(!response.contains("ERROR"));
        assert_eq!(game.board.get_cell(0, 0), Some(Cell::OpStone));

        let mut game2 = GameState::new();
        game2.handle_start(20);
        let response2 = game2.handle_turn(19, 19);
        assert!(!response2.contains("ERROR"));
        assert_eq!(game2.board.get_cell(19, 19), Some(Cell::OpStone));
    }

    #[test]
    fn test_turn_at_edge_row_0() {
        let mut game = GameState::new();
        game.handle_start(20);

        let positions = [(0, 0), (5, 0), (10, 0), (15, 0), (19, 0)];
        for (x, y) in positions {
            if game.board.get_cell(x, y) == Some(Cell::Empty) {
                let response = game.handle_turn(x, y);
                assert!(
                    !response.contains("ERROR"),
                    "Turn at ({}, {}) should succeed",
                    x,
                    y
                );
                assert_eq!(game.board.get_cell(x, y), Some(Cell::OpStone));
            }
        }
    }

    #[test]
    fn test_turn_at_edge_row_19() {
        let mut game = GameState::new();
        game.handle_start(20);

        let positions = [(0, 19), (5, 19), (10, 19), (15, 19), (19, 19)];
        for (x, y) in positions {
            if game.board.get_cell(x, y) == Some(Cell::Empty) {
                let response = game.handle_turn(x, y);
                assert!(
                    !response.contains("ERROR"),
                    "Turn at ({}, {}) should succeed",
                    x,
                    y
                );
                assert_eq!(game.board.get_cell(x, y), Some(Cell::OpStone));
            }
        }
    }

    #[test]
    fn test_game_over_win_at_edge_row_0() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..5 {
            game.board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));
    }

    #[test]
    fn test_game_over_win_at_edge_row_19() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..5 {
            game.board.set_cell(x, 19, Cell::OpStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::OpStone));
    }

    #[test]
    fn test_game_over_win_at_edge_col_0() {
        let mut game = GameState::new();
        game.handle_start(20);

        for y in 0..5 {
            game.board.set_cell(0, y, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));
    }

    #[test]
    fn test_game_over_win_at_edge_col_19() {
        let mut game = GameState::new();
        game.handle_start(20);

        for y in 0..5 {
            game.board.set_cell(19, y, Cell::OpStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::OpStone));
    }

    #[test]
    fn test_game_over_diagonal_corner_to_corner() {
        let mut game = GameState::new();
        game.handle_start(20);

        for i in 0..5 {
            game.board.set_cell(i, i, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));

        game.handle_restart();
        game.handle_start(20);

        for i in 0..5 {
            game.board.set_cell(15 + i, 15 + i, Cell::OpStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::OpStone));
    }

    #[test]
    fn test_game_over_anti_diagonal_corners() {
        let mut game = GameState::new();
        game.handle_start(20);

        for i in 0..5 {
            game.board.set_cell(19 - i, i, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));

        game.handle_restart();
        game.handle_start(20);

        for i in 0..5 {
            game.board.set_cell(4 - i, 15 + i, Cell::OpStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::OpStone));
    }

    #[test]
    fn test_game_over_no_false_positive_four() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..4 {
            game.board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), None);
    }

    #[test]
    fn test_game_over_no_false_positive_scattered() {
        let mut game = GameState::new();
        game.handle_start(20);

        game.board.set_cell(0, 0, Cell::MyStone).unwrap();
        game.board.set_cell(10, 10, Cell::MyStone).unwrap();
        game.board.set_cell(5, 5, Cell::MyStone).unwrap();
        game.board.set_cell(15, 15, Cell::MyStone).unwrap();
        game.board.set_cell(19, 19, Cell::MyStone).unwrap();
        assert_eq!(game.game_over(), None);
    }

    #[test]
    fn test_multiple_win_lines_game_over() {
        let mut game = GameState::new();
        game.handle_start(20);

        for x in 0..5 {
            game.board.set_cell(x, 0, Cell::MyStone).unwrap();
        }
        for y in 0..5 {
            game.board.set_cell(10, y, Cell::MyStone).unwrap();
        }
        assert_eq!(game.game_over(), Some(Cell::MyStone));
    }

    #[test]
    fn test_board_move_at_corners() {
        let mut game = GameState::new();
        game.handle_start(20);
        game.handle_board_start().unwrap();

        game.handle_board_move(0, 0, 1).unwrap();
        game.handle_board_move(19, 0, 2).unwrap();
        game.handle_board_move(0, 19, 1).unwrap();
        game.handle_board_move(19, 19, 2).unwrap();

        assert_eq!(game.board.get_cell(0, 0), Some(Cell::MyStone));
        assert_eq!(game.board.get_cell(19, 0), Some(Cell::OpStone));
        assert_eq!(game.board.get_cell(0, 19), Some(Cell::MyStone));
        assert_eq!(game.board.get_cell(19, 19), Some(Cell::OpStone));
    }

    #[test]
    fn test_board_move_edge_boundary() {
        let mut game = GameState::new();
        game.handle_start(20);
        game.handle_board_start().unwrap();

        assert!(game.handle_board_move(19, 0, 1).is_ok());
        assert!(game.handle_board_move(0, 19, 2).is_ok());

        assert_eq!(
            game.handle_board_move(20, 0, 1),
            Err("ERROR coordinates out of range")
        );
        assert_eq!(
            game.handle_board_move(0, 20, 1),
            Err("ERROR coordinates out of range")
        );
    }
}

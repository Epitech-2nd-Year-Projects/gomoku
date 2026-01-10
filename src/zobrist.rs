use crate::board::Cell;

const BOARD_SIZE: usize = 400;
const NUM_STONE_TYPES: usize = 2;

pub struct ZobristKeys {
    stones: [[u64; NUM_STONE_TYPES]; BOARD_SIZE],
    turn: u64,
}

impl ZobristKeys {
    pub fn new() -> Self {
        let mut keys = Self {
            stones: [[0; NUM_STONE_TYPES]; BOARD_SIZE],
            turn: 0,
        };

        let mut state = 0x853c49e6748fea9bu64;

        for pos in 0..BOARD_SIZE {
            for stone in 0..NUM_STONE_TYPES {
                state = xorshift64(state);
                keys.stones[pos][stone] = state;
            }
        }

        state = xorshift64(state);
        keys.turn = state;

        keys
    }

    #[inline]
    pub fn stone_key(&self, index: usize, cell: Cell) -> u64 {
        let stone_idx = match cell {
            Cell::MyStone => 0,
            Cell::OpStone => 1,
            _ => return 0,
        };
        self.stones[index][stone_idx]
    }

    #[inline]
    pub fn turn_key(&self) -> u64 {
        self.turn
    }
}

#[inline]
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TTFlag {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TTEntry {
    pub key: u64,
    pub score: i32,
    pub best_move: Option<(u8, u8)>,
    pub depth: u8,
    pub flag: TTFlag,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            key: 0,
            score: 0,
            best_move: None,
            depth: 0,
            flag: TTFlag::Exact,
        }
    }
}

const TT_SIZE: usize = 1 << 22; // 4,194,304 entries (~67MB)

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: vec![TTEntry::default(); TT_SIZE],
        }
    }

    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (TT_SIZE - 1)
    }

    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = self.index(hash);
        let entry = &self.entries[idx];
        if entry.key == hash {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        score: i32,
        flag: TTFlag,
        best_move: Option<(usize, usize)>,
    ) {
        let idx = self.index(hash);
        let existing = &self.entries[idx];

        if existing.key == 0 || existing.depth <= depth {
            self.entries[idx] = TTEntry {
                key: hash,
                score,
                best_move: best_move.map(|(x, y)| (x as u8, y as u8)),
                depth,
                flag,
            };
        }
    }

    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = TTEntry::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zobrist_keys_deterministic() {
        let keys1 = ZobristKeys::new();
        let keys2 = ZobristKeys::new();

        assert_eq!(keys1.stones[0][0], keys2.stones[0][0]);
        assert_eq!(keys1.stones[399][1], keys2.stones[399][1]);
        assert_eq!(keys1.turn, keys2.turn);
    }

    #[test]
    fn test_zobrist_keys_unique() {
        let keys = ZobristKeys::new();

        assert_ne!(keys.stones[0][0], keys.stones[0][1]);
        assert_ne!(keys.stones[0][0], keys.stones[1][0]);
        assert_ne!(keys.stones[0][0], keys.turn);
    }

    #[test]
    fn test_tt_store_and_probe() {
        let mut tt = TranspositionTable::new();
        let hash = 0x123456789abcdef0u64;

        tt.store(hash, 5, 100, TTFlag::Exact, Some((10, 10)));

        let entry = tt.probe(hash).expect("Entry should exist");
        assert_eq!(entry.key, hash);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 100);
        assert_eq!(entry.flag, TTFlag::Exact);
        assert_eq!(entry.best_move, Some((10, 10)));
    }

    #[test]
    fn test_tt_probe_miss() {
        let tt = TranspositionTable::new();
        assert!(tt.probe(0x123456789abcdef0u64).is_none());
    }

    #[test]
    fn test_tt_depth_replacement() {
        let mut tt = TranspositionTable::new();
        let hash = 0x123456789abcdef0u64;

        tt.store(hash, 3, 50, TTFlag::LowerBound, None);
        tt.store(hash, 5, 100, TTFlag::Exact, Some((5, 5)));

        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 100);
    }
}

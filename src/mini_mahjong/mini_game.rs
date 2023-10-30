use std::collections::HashMap;
use std::fmt;

// tile is represented as a number 0-35 (we only have one suit, ranks 1 to 9, and four copies of each rank)
pub const NUM_MINI_TILES: u32 = 4 * 9;
// hand : list of tiles

#[derive(Clone, Copy)]
pub struct MiniTile {
    pub serial: u32,
}

impl fmt::Display for MiniTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.rank())
    }
}

impl fmt::Debug for MiniTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MiniTile {} (serial={})", self.rank(), self.serial)
    }
}

impl MiniTile {
    // TODO use this function when initializing tiles via serial number?
    pub fn is_valid_serial(&self) -> bool {
        self.serial < NUM_MINI_TILES
    }

    /// The rank (aka value) of the tile
    pub fn rank(&self) -> u32 {
        (self.serial % 9) + 1
    }
}

fn count_mini_tiles_by_rank(tiles: &Vec<MiniTile>) -> HashMap<u32, u32> {
    let mut tile_counts_by_rank: HashMap<u32, u32> = HashMap::new();
    for tile in tiles.iter() {
        let rank = tile.rank();
        let count = tile_counts_by_rank.entry(rank).or_insert(0);
        *count += 1;
    }
    tile_counts_by_rank
}

fn contains_sequence(tile_counts: &HashMap<u32, u32>) -> bool {
    for rank in 1u32..=9 {
        if tile_counts.get(&rank).unwrap_or(&0) > &0 {
            if rank > 7 {
                return false;
            }
            let second_rank_count = tile_counts.get(&(rank + 1)).unwrap_or(&0);
            let third_rank_count = tile_counts.get(&(rank + 2)).unwrap_or(&0);
            if second_rank_count > &0 && third_rank_count > &0 {
                return true;
            }
        }
    }
    false
}

fn contains_triplet(tile_counts: &HashMap<u32, u32>) -> bool {
    for (_rank, count) in tile_counts {
        if count >= &3 {
            return true;
        }
    }
    false
}

pub fn is_winning_mini_hand(tiles: &Vec<MiniTile>) -> bool {
    // println!(
    //     "check for winning hand {}",
    //     simulator::display_hand(&new_tiles)
    // );
    if tiles.len() != 5 {
        // invalid number of tiles for winning mini hand
        return false;
    }

    let tile_counts = count_mini_tiles_by_rank(tiles);

    for (&rank, count) in &tile_counts {
        if count >= &2 {
            // potential pair
            let mut new_tile_counts = tile_counts.clone();
            let count = new_tile_counts.entry(rank).or_insert(0);
            *count -= 2;
            let new_tile_counts = new_tile_counts;
            if contains_sequence(&new_tile_counts) || contains_triplet(&new_tile_counts) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_mini_tile_rank() {
        assert_eq!(MiniTile { serial: 0 }.rank(), 1);
        assert_eq!(MiniTile { serial: 3 }.rank(), 4);
        assert_eq!(MiniTile { serial: 8 }.rank(), 9);
        assert_eq!(MiniTile { serial: 9 }.rank(), 1);
        assert_eq!(MiniTile { serial: 10 }.rank(), 2);
        assert_eq!(MiniTile { serial: 35 }.rank(), 9);
    }

    #[test]
    fn test_count_mini_tiles_by_rank() {
        let tiles = vec![
            MiniTile { serial: 0 },  // 1p
            MiniTile { serial: 9 },  // 1p
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 34 }, // 8p
            MiniTile { serial: 6 },  // 7p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(tile_counts.get(&1), Some(&2));
        assert_eq!(tile_counts.get(&2), Some(&1));
        assert_eq!(tile_counts.get(&8), Some(&1));
        assert_eq!(tile_counts.get(&7), Some(&1));
        assert_eq!(tile_counts.get(&9), None);
    }

    #[test]
    fn test_contains_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 4 }, // 5p
            MiniTile { serial: 5 }, // 6p
            MiniTile { serial: 6 }, // 7p
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 1 }, // 2p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_sequence(&tile_counts), true);
    }

    #[test]
    fn test_contains_edge_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 6 }, // 7p
            MiniTile { serial: 7 }, // 8p
            MiniTile { serial: 8 }, // 9p
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 1 }, // 2p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_sequence(&tile_counts), true);
    }

    #[test]
    fn test_sequence_cannot_wrap() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 7 }, // 8p
            MiniTile { serial: 8 }, // 9p
            MiniTile { serial: 9 }, // 1p
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 0 }, // 1p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_sequence(&tile_counts), false);
    }

    #[test]
    fn test_does_not_contain_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 0 }, // 1p
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 3 }, // 4p
            MiniTile { serial: 4 }, // 5p
            MiniTile { serial: 6 }, // 7p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_sequence(&tile_counts), false);
    }

    #[test]
    fn test_contains_triplet() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 0 },  // 1p
            MiniTile { serial: 9 },  // 1p
            MiniTile { serial: 18 }, // 1p
            MiniTile { serial: 4 },  // 5p
            MiniTile { serial: 6 },  // 7p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_triplet(&tile_counts), true);
    }

    #[test]
    fn test_does_not_contain_triplet() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 0 },  // 1p
            MiniTile { serial: 9 },  // 1p
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 19 }, // 2p
            MiniTile { serial: 8 },  // 9p
        ];

        let tile_counts = count_mini_tiles_by_rank(&tiles);
        assert_eq!(contains_triplet(&tile_counts), false);
    }

    #[test]
    fn test_winning_mini_hand_with_triplet() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 0 },  // 1p
            MiniTile { serial: 9 },  // 1p
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 19 }, // 2p
            MiniTile { serial: 28 }, // 2p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), true);
    }

    #[test]
    fn test_winning_mini_hand_with_triplet2() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 0 },  // 1p
            MiniTile { serial: 9 },  // 1p
            MiniTile { serial: 3 },  // 4p
            MiniTile { serial: 12 }, // 4p
            MiniTile { serial: 21 }, // 4p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), true);
    }

    #[test]
    fn test_winning_mini_hand_with_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 7 },  // 8p
            MiniTile { serial: 16 }, // 8p
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 11 }, // 3p
            MiniTile { serial: 3 },  // 4p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), true);
    }

    #[test]
    fn test_winning_mini_hand_pair_in_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 4 },  // 5p
            MiniTile { serial: 13 }, // 5p
            MiniTile { serial: 22 }, // 5p
            MiniTile { serial: 2 },  // 3p
            MiniTile { serial: 3 },  // 4p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), true);
    }

    #[test]
    fn test_not_winning_mini_hand_missing_pair() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 2 }, // 3p
            MiniTile { serial: 3 }, // 4p
            MiniTile { serial: 4 }, // 5p
            MiniTile { serial: 5 }, // 6p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), false);
    }

    #[test]
    fn test_not_winning_mini_hand_despite_sequence() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 2 },  // 3p
            MiniTile { serial: 11 }, // 3p
            MiniTile { serial: 3 },  // 4p
            MiniTile { serial: 4 },  // 5p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), false);
    }

    #[test]
    fn test_not_winning_mini_hand_despite_triplet() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 2 },  // 3p
            MiniTile { serial: 11 }, // 3p
            MiniTile { serial: 20 }, // 3p
            MiniTile { serial: 4 },  // 5p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), false);
    }

    #[test]
    fn test_not_winning_mini_hand_too_few_tiles() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 1 }, // 2p
            MiniTile { serial: 2 }, // 3p
            MiniTile { serial: 3 }, // 4p
            MiniTile { serial: 4 }, // 5p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), false);
    }

    #[test]
    fn test_not_winning_mini_hand_too_many_tiles() {
        let tiles: Vec<MiniTile> = vec![
            MiniTile { serial: 1 },  // 2p
            MiniTile { serial: 2 },  // 3p
            MiniTile { serial: 3 },  // 4p
            MiniTile { serial: 4 },  // 5p
            MiniTile { serial: 13 }, // 5p
            MiniTile { serial: 22 }, // 5p
        ];

        assert_eq!(is_winning_mini_hand(&tiles), false);
    }
}

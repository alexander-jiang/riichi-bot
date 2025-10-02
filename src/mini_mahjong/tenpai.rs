use crate::mini_mahjong::mini_game::{is_winning_mini_hand, MiniTile};
// use crate::mini_mahjong::simulator;

pub fn get_tenpai_tiles(tiles: &Vec<MiniTile>) -> Vec<MiniTile> {
    if tiles.len() != 4 {
        panic!("too few tiles!");
    }

    // println!(
    //     "are these tiles in tenpai: {}",
    //     simulator::display_hand(&tiles)
    // );

    // try adding each of the possible tiles and check if the result is a winning hand
    let mut winning_tiles: Vec<MiniTile> = Vec::new();
    for new_tile_rank in 1..=9 {
        // if in tenpai, returns a list of winning tiles
        let mut new_tiles = tiles.clone();
        // println!("check tenpai if adding tile rank {}", new_tile_rank);
        let new_tile = MiniTile {
            serial: new_tile_rank - 1,
        };
        new_tiles.push(new_tile);

        if is_winning_mini_hand(&new_tiles) {
            // println!("found new winning tile {}", new_tile);
            winning_tiles.push(new_tile);
        }
    }
    let winning_tiles = winning_tiles;
    // if not in tenpai, returns empty list
    winning_tiles
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tenpai_tanki_with_triplet() {
        // tiles: 1118p - wins on 8
        let tiles = vec![
            MiniTile { serial: 0 },
            MiniTile { serial: 9 },
            MiniTile { serial: 18 },
            MiniTile { serial: 7 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 1);
        assert_eq!(
            tenpai_tiles.get(0).expect("Expected a winning tile").rank(),
            8
        );
    }

    #[test]
    fn test_tenpai_tanki_with_sequence() {
        // tiles: 3458p - wins on 8
        let tiles = vec![
            MiniTile { serial: 2 },
            MiniTile { serial: 3 },
            MiniTile { serial: 4 },
            MiniTile { serial: 7 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 1);
        assert_eq!(
            tenpai_tiles.get(0).expect("Expected a winning tile").rank(),
            8
        );
    }

    #[test]
    fn test_tenpai_kanchan() {
        // tiles: 2257p - wins on 6
        let tiles = vec![
            MiniTile { serial: 1 },
            MiniTile { serial: 10 },
            MiniTile { serial: 4 },
            MiniTile { serial: 6 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 1);
        assert_eq!(
            tenpai_tiles.get(0).expect("Expected a winning tile").rank(),
            6
        );
    }

    #[test]
    fn test_tenpai_penchan() {
        // tiles: 4489p - wins on 7
        let tiles = vec![
            MiniTile { serial: 3 },
            MiniTile { serial: 12 },
            MiniTile { serial: 7 },
            MiniTile { serial: 8 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 1);
        assert_eq!(
            tenpai_tiles.get(0).expect("Expected a winning tile").rank(),
            7
        );
    }

    #[test]
    fn test_tenpai_ryanmen() {
        // tiles: 3378p - wins on 6, 9
        let tiles = vec![
            MiniTile { serial: 2 },
            MiniTile { serial: 11 },
            MiniTile { serial: 6 },
            MiniTile { serial: 7 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&6));
        assert!(tenpai_tile_ranks.contains(&9));
    }

    #[test]
    fn test_tenpai_aryanmen() {
        // tiles: 5677p - wins on 4, 7
        let tiles = vec![
            MiniTile { serial: 4 },
            MiniTile { serial: 5 },
            MiniTile { serial: 6 },
            MiniTile { serial: 15 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&4));
        assert!(tenpai_tile_ranks.contains(&7));
    }

    #[test]
    fn test_tenpai_shanpon() {
        // tiles: 2277p - wins on 2, 7
        let tiles = vec![
            MiniTile { serial: 1 },
            MiniTile { serial: 10 },
            MiniTile { serial: 6 },
            MiniTile { serial: 15 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&2));
        assert!(tenpai_tile_ranks.contains(&7));
    }

    #[test]
    fn test_tenpai_nobetan() {
        // tiles: 2345p - wins on 2, 5
        let tiles = vec![
            MiniTile { serial: 1 },
            MiniTile { serial: 2 },
            MiniTile { serial: 3 },
            MiniTile { serial: 4 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&2));
        assert!(tenpai_tile_ranks.contains(&5));
    }

    #[test]
    fn test_tenpai_kantan() {
        // tiles: 2224p - wins on 3, 4
        let tiles = vec![
            MiniTile { serial: 1 },
            MiniTile { serial: 10 },
            MiniTile { serial: 19 },
            MiniTile { serial: 3 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&3));
        assert!(tenpai_tile_ranks.contains(&4));
    }

    #[test]
    fn test_tenpai_pentan() {
        // tiles: 8889p - wins on 7, 9
        let tiles = vec![
            MiniTile { serial: 7 },
            MiniTile { serial: 16 },
            MiniTile { serial: 25 },
            MiniTile { serial: 8 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 2);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&7));
        assert!(tenpai_tile_ranks.contains(&9));
    }

    #[test]
    fn test_tenpai_ryantan() {
        // tiles: 4445p - wins on 3, 5, 6
        let tiles = vec![
            MiniTile { serial: 3 },
            MiniTile { serial: 12 },
            MiniTile { serial: 21 },
            MiniTile { serial: 4 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 3);
        let tenpai_tile_ranks: Vec<u32> = tenpai_tiles.iter().map(|tile| tile.rank()).collect();
        assert!(tenpai_tile_ranks.contains(&3));
        assert!(tenpai_tile_ranks.contains(&5));
        assert!(tenpai_tile_ranks.contains(&6));
    }

    #[test]
    fn test_tenpai_noten() {
        // tiles: 3478p - not in tenpai (doesn't win off of a single tile)
        let tiles = vec![
            MiniTile { serial: 2 },
            MiniTile { serial: 3 },
            MiniTile { serial: 6 },
            MiniTile { serial: 7 },
        ];
        let tenpai_tiles = get_tenpai_tiles(&tiles);
        assert_eq!(tenpai_tiles.len(), 0);
    }
}

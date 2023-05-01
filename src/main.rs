// pub mod tiles;
// use crate::tiles::Tile;

use std::collections::HashMap;

// const NUMBER_TILES: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
// const WIND_TILES: [&str; 4] = ["E", "S", "W", "N"];
// const DRAGON_TILES: [&str; 3] = ["G", "R", "W"];
// const TILE_SUITS: [&str; 5] = ["m", "p", "s", "w", "d"];
const HONOR_SUITS: [&str; 2] = ["w", "d"];
// const NUMBER_SUITS: [&str; 3] = ["m", "p", "s"];

// fn check_valid_tile_string(tile_str: &String) {
//     assert!(tile_str.len() == 2);
//     let tile_rank = tile_str.get(0..1).expect("Expected tile rank");
//     let tile_suit = tile_str.get(1..2).expect("Expected tile suit");
//     assert!(
//         NUMBER_TILES.contains(&tile_rank)
//             || WIND_TILES.contains(&tile_rank)
//             || DRAGON_TILES.contains(&tile_rank)
//     );
//     assert!(TILE_SUITS.contains(&tile_suit));
// }

fn is_winning_hand(tiles: Vec<String>) -> bool {
    // check hand length, must be minimum 14 tiles (could be more if there are quads)
    if tiles.len() < 14 {
        return false;
    }

    // TODO check for edge case hands: 7 pairs and 13 orphans

    // build frequency count per tile suit: mapping of tile suit -> (mapping of rank -> count)
    let mut tile_counts_by_suit: HashMap<String, HashMap<String, i32>> = HashMap::new();
    for tile_str in tiles {
        let tile_rank = String::from(tile_str.get(0..1).expect("Expected tile rank"));
        let tile_suit = String::from(tile_str.get(1..2).expect("Expected tile suit"));

        if tile_counts_by_suit.contains_key(&tile_suit) {
            let suit_counts = tile_counts_by_suit.get_mut(&tile_suit).unwrap();
            let count = suit_counts.entry(tile_rank).or_insert(0);
            *count += 1;
        } else {
            let mut empty_counts = HashMap::new();
            // we have to update/mutate `empty_counts` before we call `tile_counts_by_suit.insert(...)`,
            //  as that insert will transfer ownership of `empty_counts`
            empty_counts.insert(tile_rank, 1);
            tile_counts_by_suit.insert(tile_suit, empty_counts);
        }
    }
    println!("{:?}", tile_counts_by_suit);

    // there can be at most one pair
    let mut pair_tile = String::from("");

    // check honor tiles:
    // - any isolated honors? if so, not winning
    // - if there is a pair, that must be the only pair in the hand
    for (tile_suit, suit_counts) in tile_counts_by_suit {
        if !HONOR_SUITS.contains(&(tile_suit.as_str())) {
            continue;
        }
        for (tile_rank, tile_count) in suit_counts {
            let mut new_tile_str = String::new();
            new_tile_str.push_str(tile_rank.as_str());
            new_tile_str.push_str(tile_suit.as_str());

            if tile_count == 1 {
                // isolated honor tile
                println!("isolated honor tile {new_tile_str}");
                return false;
            } else if tile_count == 2 {
                if !pair_tile.is_empty() {
                    // honor tile must be the pair, but can only have one pair in the winning hand
                    println!("pair of honor tile {new_tile_str} but already have a pair");
                    return false;
                } else {
                    pair_tile = new_tile_str;
                }
            }
        }
    }

    println!("pair tile so far = {}", pair_tile);

    // TODO check numbered suits:
    // - start from the lowest number in the suit
    // - there must be at least 2 tiles of that number, or a tile of the next higher number
    // -

    return false;
}

fn main() {
    // let tile = Tile { suit: TileSuit::Man, rank: NumberTileType::Five};
    // println!("It's a tile: {}", tile.repr());

    let mut hand: Vec<String> = Vec::new();
    hand.push(String::from("1s"));
    hand.push(String::from("2s"));
    hand.push(String::from("3s"));
    hand.push(String::from("4s"));
    hand.push(String::from("5s"));
    hand.push(String::from("6s"));
    hand.push(String::from("7s"));
    hand.push(String::from("8s"));
    hand.push(String::from("9s"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Nw"));
    hand.push(String::from("Nw"));
    println!("hand:");
    for tile in &hand {
        println!("{tile}");
    }
    println!("is winning hand? {}", is_winning_hand(hand));
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_winning_hand_half_flush() {
        // from: https://riichi.wiki/Honiisou
        let tiles = Vec::from([
            String::from("1m"),
            String::from("1m"),
            String::from("1m"),
            String::from("2m"),
            String::from("3m"),
            String::from("4m"),
            String::from("8m"),
            String::from("8m"),
            String::from("Gd"),
            String::from("Gd"),
            String::from("Ww"),
            String::from("Ww"),
            String::from("Ww"),
        ]);

        for winning_tile in [String::from("8m"), String::from("Gd")] {
            let mut new_tiles = Vec::new();
            new_tiles.clone_from_slice(tiles.as_slice());
            new_tiles.push(winning_tile);
            assert_eq!(is_winning_hand(new_tiles), true);
        }
        assert_eq!(is_winning_hand(tiles), false);
    }

    #[test]
    fn test_is_winning_hand_isolated_honor() {
        let tiles = Vec::from([
            String::from("1m"),
            String::from("1m"),
            String::from("1m"),
            String::from("2m"),
            String::from("3m"),
            String::from("4m"),
            String::from("5m"),
            String::from("6m"),
            String::from("7m"),
            String::from("8m"),
            String::from("9m"),
            String::from("9m"),
            String::from("9m"),
            String::from("Ww"), // isolated honor tile
        ]);
        assert_eq!(is_winning_hand(tiles), false);
    }

    #[test]
    fn test_is_winning_hand_isolated_number() {
        let tiles = Vec::from([
            String::from("1m"), // isolated number tile
            String::from("3m"),
            String::from("4m"),
            String::from("5m"),
            String::from("6m"),
            String::from("7m"),
            String::from("8m"),
            String::from("9m"),
            String::from("2s"),
            String::from("2s"),
            String::from("2s"),
            String::from("Rd"),
            String::from("Rd"),
            String::from("Rd"),
        ]);
        assert_eq!(is_winning_hand(tiles), false);
    }

    #[test]
    fn test_is_winning_hand_nine_gates() {
        let tiles = Vec::from([
            String::from("1m"),
            String::from("1m"),
            String::from("1m"),
            String::from("2m"),
            String::from("3m"),
            String::from("4m"),
            String::from("5m"),
            String::from("6m"),
            String::from("7m"),
            String::from("8m"),
            String::from("9m"),
            String::from("9m"),
            String::from("9m"),
        ]);
        for rank in 1..=9 {
            let mut new_tiles = Vec::new();
            new_tiles.clone_from_slice(tiles.as_slice());
            let mut added_tile = String::from(rank.to_string());
            added_tile.push('m');
            new_tiles.push(added_tile);
            assert_eq!(is_winning_hand(new_tiles), true);
        }
    }
}

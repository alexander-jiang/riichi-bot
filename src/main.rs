// pub mod tiles;
// use crate::tiles::Tile;

use std::collections::HashMap;

fn check_valid_tile_string(tile_str: &String) {
    assert!(tile_str.len() == 2);
    assert!(
        tile_str.starts_with('1')
            || tile_str.starts_with('2')
            || tile_str.starts_with('3')
            || tile_str.starts_with('4')
            || tile_str.starts_with('5')
            || tile_str.starts_with('6')
            || tile_str.starts_with('7')
            || tile_str.starts_with('8')
            || tile_str.starts_with('9')
            || tile_str.starts_with('E')
            || tile_str.starts_with('S')
            || tile_str.starts_with('W')
            || tile_str.starts_with('N')
            || tile_str.starts_with('G')
            || tile_str.starts_with('R')
    );
    assert!(
        tile_str.ends_with('m')
            || tile_str.ends_with('p')
            || tile_str.ends_with('s')
            || tile_str.ends_with('w')
            || tile_str.ends_with('d')
    );
}

fn is_honor_tile(tile_str: &String) -> bool {
    check_valid_tile_string(tile_str);
    tile_str.ends_with('w') || tile_str.ends_with('d')
}

fn is_winning_hand(tiles: Vec<String>) -> bool {
    // check hand length, must be minimum 14 tiles (could be more if there are quads)
    if tiles.len() < 14 {
        return false;
    }

    // build frequency count mapping
    let mut tile_counts = HashMap::new();
    for tile_str in tiles {
        let count = tile_counts.entry(tile_str).or_insert(0);
        *count += 1;
    }
    println!("{:?}", tile_counts);

    // TODO check honors
    // any isolated honors? if so, not winning
    // if there is a pair, that must be the only pair in the handsrc/

    // TODO then check numbered suits
    // start from the lowest number in the suit

    return false;
}

fn main() {
    // let tile = Tile { suit: TileSuit::Man, rank: NumberTileType::Five};
    // println!("It's a tile: {}", tile.repr());
    let tile = String::from("4s");
    println!("is 4s honor tile? {}", is_honor_tile(&tile));
    let wind_tile = String::from("Ww");
    println!("is WW honor tile? {}", is_honor_tile(&wind_tile));

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

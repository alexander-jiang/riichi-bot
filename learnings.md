- which is better, vec![] or Vec::new()?

- understanding the difference between Rust for loops on Vec: https://stackoverflow.com/a/66909084

  - iterating on the Vec directly (e.g. `for e in vector`): you will consume the vector and its elements (move each value out of the vector)
  - iterating on Vec::iter (e.g. `for e in vector.iter()`): you iterate on immutable references to the elements in the vector
  - iterating on Vec::iter_mut (e.g. `for e in vector.iter_mut()`): you iterate on mutable references to the elements in the vector

- Why doesn't the following function work?

```rust
// Why doesn't the following function work?
pub fn tiles_to_count_array(tiles_string: &str) -> [u8; 34] {

    let mut start_index = 0;
    while start_index < tiles_string.len() {
        let substr = &tiles_string[start_index..];
        let next_suit_index = substr.find(&['m', 's', 'p', 'z']);
        match next_suit_index {
            Some(suit_idx) => {
                let suit_char = substr[suit_idx..].chars().next().unwrap();
                println!("found suit {} at char index {}", suit_char, suit_idx);
                for idx in start_index..suit_idx {
                    let mut tile_string = String::new();
                    let rank_char = substr[idx..].chars().next().unwrap();
                    println!("found tile {}{}", rank_char, suit_char);
                    tile_string.push(rank_char);
                    tile_string.push(suit_char);
                    let tile_id = mahjong_tile::get_id_from_tile_text(&tile_string).unwrap();
                    tile_count_array[usize::from(tile_id)] += 1;
                }
                start_index = suit_idx + 1;
            }
            None => panic!(
                "expected to find a suit indicator after index {}",
                start_index
            ),
        }
    }
    tile_count_array
}
```

- the hand scoring on this website isn't always accurate e.g. https://riichi.harphield.com/tools/hand-analyzer/?hand=55588m11p234s666z1p - this hand "55588m11p234s666z1p" (when winning by ron) should be scored as 1 han, 50 fu (20 + 10 fu for closed ron + 8 fu for ankou of 6z/green dragon + 4 fu for ankou of 5m + 4 fu for minkou of 1p, since the tile was won by ron), e.g. https://mahjongo.com/tools/riichi-calculator, but was incorrectly scored as 1 han, 60 fu instead.

- the upgrade tiles isn't always complete on the [efficiency trainer](https://euophrys.itch.io/mahjong-efficiency-trainer) "Explorer" page, for example: hand 5789s57p34667m111z (after calling pon on 1z) - if discard 5s -> 1 shanten with 12 ukiere (25m6p). The efficiency trainer is correct so far. However, the tiles listed as upgrades are 6m (draw 6m, discard 7m: 24 ukiere) and 8m (draw 8m, discard 6m: 24 ukiere) -- these are correct, but are not the only upgrade tiles. For example, after drawing 3m (334667m57p789s111z), you can discard 4m or 7m for 1 shanten with 16 ukiere (3568m6p or 2356m6p, respectively). In total, the upgrade options are (in addition to 68m): 34m345789p

- learned about creating a `hashmap!` macro to instantiate `HashMap` objects in-line: https://stackoverflow.com/a/41143449

- other websites:
  - https://euophrys.itch.io/mahjong-efficiency-trainer
    - source code: https://github.com/Euophrys/Riichi-Trainer/blob/develop/src/scripts/ShantenCalculator.js#L157
  - https://mahjong-trainer.netlify.app/
    - https://github.com/djuretic/riichi-mahjong-trainer/blob/main/tests/ShantenTest.elm
  - https://tenhou.net/2/?q=12234558s345p11z4s

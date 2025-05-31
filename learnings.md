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

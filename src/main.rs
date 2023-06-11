pub mod state;
pub mod tiles;
pub mod yaku;
pub mod tile_grouping;

fn main() {
    for serial in 0..tiles::NUM_TILES {
        let tile = tiles::Tile { serial };
        // print!("{} ", tile.to_string());
        print!("{} ", tile.to_human_string());
        if (serial < 3 * 36 && serial % 9 == 8) || (serial >= 3 * 36 && (serial - 3 * 36) % 7 == 6)
        {
            println!("");
        }
    }
}

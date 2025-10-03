extern crate test;

pub use crate::mahjong_tile;
use crate::mahjong_tile::{MahjongTileCountArray, MahjongTileId, FOUR_OF_EACH_TILE_COUNT_ARRAY};
pub use crate::shanten;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

// 345m11256p46778s6s
// first start with 1-shanten hands:
// e.g. discard 4s vs. discard 2p
// 1. naive analysis, no upgrades - assuming all 136 tiles are available, with replacement
// 1a. like above, but take upgrades when possible. (instead of only accepting ukiere, we also account for upgrade tiles)
// 2. level 1 analysis - assuming all tiles outside of the ones used in the hand are available, without replacement
// 2a. like above, but take upgrades when possible. (instead of only accepting ukiere, we also account for upgrade tiles)
// 3. level 2 analysis - accounting for all visible tiles so far (discarded so far + dora indicator + any declared melds)
// 3a. like above, but take upgrades when possible. (instead of only accepting ukiere, we also account for upgrade tiles)
// 4. level 3 analysis - accounting for furiten

/// generates a random tile id from the pool of tiles remaining (panics if there are no tiles left),
/// weighted by the number of copies of tiles remaining
#[allow(unused)]
fn generate_random_tile_id_rng(remaining_tile_count: MahjongTileCountArray) -> MahjongTileId {
    let tiles_remaining = mahjong_tile::get_tile_ids_from_count_array(remaining_tile_count);
    if tiles_remaining.len() == 0 {
        panic!("no tiles remaining");
    }
    let mut rng = rand::rng();
    let n: usize = rng.random_range(0..tiles_remaining.len());
    *tiles_remaining.get(n).unwrap()
}

/// generates a random tile id from the pool of tiles remaining (panics if there are no tiles left),
/// weighted by the number of copies of tiles remaining
#[allow(unused)]
fn generate_random_tile_id(
    remaining_tile_count: MahjongTileCountArray,
    rng: &mut ThreadRng,
) -> MahjongTileId {
    let tiles_remaining = mahjong_tile::get_tile_ids_from_count_array(remaining_tile_count);
    if tiles_remaining.len() == 0 {
        panic!("no tiles remaining");
    }
    let n: usize = rng.random_range(0..tiles_remaining.len());
    *tiles_remaining.get(n).unwrap()
}

fn remove_tile_ids_from_count_array<T: Into<MahjongTileId> + Clone>(
    tile_count_array: MahjongTileCountArray,
    tile_ids_to_remove: &Vec<T>,
) -> MahjongTileCountArray {
    let mut tiles_after_remove: MahjongTileCountArray = Default::default();
    for i in 0..tiles_after_remove.0.len() {
        tiles_after_remove.0[i] = tile_count_array.0[i];
    }
    for tile_id in tile_ids_to_remove {
        let tile_id: MahjongTileId = tile_id.clone().into();
        if tiles_after_remove.0[usize::from(tile_id)] == 0 {
            panic!("no more copies of tile left to remove");
        }
        tiles_after_remove.0[usize::from(tile_id)] -= 1;
    }
    tiles_after_remove
}

/// takes a starting hand (after discard), any visible tiles outside of the starting hand, and the parameters for the simulation:
/// number of trials and maximum allowed draws per trial
#[allow(unused)]
fn run_basic_analysis<T: Into<MahjongTileId> + Clone>(
    starting_hand: MahjongTileCountArray,
    visible_tile_ids: &Vec<T>,
    num_trials: u32,
    max_allowed_draws: u16,
) {
    let starting_shanten = shanten::get_shanten_optimized(starting_hand);
    if starting_shanten != 1 {
        todo!("only implemented for 1-shanten hands so far");
    }
    let starting_ukiere_tile_ids = shanten::get_ukiere_optimized(starting_hand);
    let starting_hand_tile_ids = mahjong_tile::get_tile_ids_from_count_array(starting_hand);
    let total_num_visible_tiles =
        visible_tile_ids.len() + mahjong_tile::get_total_tiles_from_count_array(starting_hand);
    println!(
        "initial hand: {}",
        shanten::tile_ids_to_string(&starting_hand_tile_ids)
    );
    println!(
        "initial ukiere tiles: {}",
        shanten::tile_ids_to_string(&starting_ukiere_tile_ids)
    );
    println!(
        "total num tiles visible initially: {}",
        total_num_visible_tiles
    );
    println!(
        "initially visible tiles: {}",
        shanten::tile_ids_to_string(&visible_tile_ids)
    );

    // precompute (discard tile id, resulting ukiere tile ids) after each possible improvement tile draw
    let mut draw_tile_id_to_ukiere_tile_ids: HashMap<
        MahjongTileId,
        Vec<(MahjongTileId, Vec<MahjongTileId>, u16)>,
    > = HashMap::new();
    for improve_tile_id in starting_ukiere_tile_ids.iter() {
        let improve_tile_id = *improve_tile_id;
        let mut tile_count_array_after_draw = starting_hand;
        if tile_count_array_after_draw.0[usize::from(improve_tile_id)] == 4 {
            panic!(
                "cannot draw more than a fourth copy of the tile {}",
                mahjong_tile::get_tile_text_from_id(improve_tile_id)
            );
        }
        tile_count_array_after_draw.0[usize::from(improve_tile_id)] += 1;
        let ukiere_after_improve = shanten::get_all_ukiere_after_discard(
            tile_count_array_after_draw,
            starting_shanten - 1,
            &shanten::get_shanten_optimized,
            &shanten::get_ukiere_optimized,
            &visible_tile_ids,
        );
        draw_tile_id_to_ukiere_tile_ids.insert(improve_tile_id, ukiere_after_improve);
    }

    let mut num_times_reached_tenpai: u32 = 0;
    let mut total_num_turns_to_reach_tenpai: u32 = 0;
    let mut total_num_ukiere_at_tenpai: u32 = 0;
    let mut count_turns_reached_tenpai = vec![0; usize::from(max_allowed_draws)];
    for _iter_number in 1..=num_trials {
        // in this case, we assume all non-visible are available, without replacement
        if _iter_number % 10000 == 0 {
            println!("trial {}", _iter_number);
        }
        // start the trial: set up initial remaining tile count (remove visible tiles and the tiles from the starting hand)
        let mut total_visible_tiles_so_far =
            shanten::combine_tile_ids_from_count_array_and_vec(starting_hand, visible_tile_ids);
        // println!(
        //     "total visible tiles so far: {}",
        //     shanten::tile_ids_to_string(&total_visible_tiles_so_far)
        // );
        let mut remaining_tile_count = remove_tile_ids_from_count_array(
            FOUR_OF_EACH_TILE_COUNT_ARRAY,
            &total_visible_tiles_so_far,
        );

        let mut hand: MahjongTileCountArray = Default::default();
        for tile_id in 0..starting_hand.0.len() {
            hand.0[tile_id] = starting_hand.0[tile_id];
        }

        // draw a tile
        for draw_number in 1..=max_allowed_draws {
            let drawn_tile_id = generate_random_tile_id_rng(remaining_tile_count);
            // let hand_tile_ids = mahjong_tile::get_tile_ids_from_count_array(hand);
            // println!(
            //     "draw {}, hand = {}",
            //     mahjong_tile::get_tile_text_from_id(drawn_tile_id),
            //     shanten::tile_ids_to_string(&hand_tile_ids)
            // );
            if remaining_tile_count.0[usize::from(drawn_tile_id)] == 0 {
                panic!("no more copies of tile left to remove");
            }
            remaining_tile_count.0[usize::from(drawn_tile_id)] -= 1;
            total_visible_tiles_so_far.push(drawn_tile_id);

            // in this case, we only accept ukiere tiles, disregard upgrades
            if starting_ukiere_tile_ids.contains(&drawn_tile_id) {
                let ukiere_options = draw_tile_id_to_ukiere_tile_ids.get(&drawn_tile_id).unwrap();
                let mut max_num_ukiere_after_improve_discard = 0;
                for (_discard_tile_id, ukiere_after_improve_discard, _) in ukiere_options {
                    // the drawn tile is in visible tiles, and
                    // the discarded tile is already in visible tiles (since it comes from the hand, which was already included)
                    let actual_num_ukiere_after_improve_discard = shanten::get_num_tiles_remaining(
                        &ukiere_after_improve_discard,
                        &total_visible_tiles_so_far,
                    );
                    // println!(
                    //     "can discard {} , resulting ukiere: {} tiles, {}",
                    //     mahjong_tile::get_tile_text_from_id(*_discard_tile_id),
                    //     actual_num_ukiere_after_improve_discard,
                    //     shanten::tile_ids_to_string(ukiere_after_improve_discard)
                    // );
                    // println!(
                    //     "visible tiles after discard: {}",
                    //     shanten::tile_ids_to_string(&total_visible_tiles_so_far)
                    // );
                    if actual_num_ukiere_after_improve_discard
                        > max_num_ukiere_after_improve_discard
                    {
                        max_num_ukiere_after_improve_discard =
                            actual_num_ukiere_after_improve_discard;
                    }
                }
                // println!(
                //     "reached tenpai after {} draws, {} ukiere at tenpai:",
                //     draw_number, max_num_ukiere_after_improve_discard
                // );
                // shanten::print_ukiere_after_discard(&ukiere_after_improve);

                num_times_reached_tenpai += 1;
                total_num_turns_to_reach_tenpai += u32::from(draw_number);
                let draw_number_index = draw_number - 1;
                *count_turns_reached_tenpai
                    .get_mut(usize::from(draw_number_index))
                    .unwrap() += 1;
                total_num_ukiere_at_tenpai += u32::from(max_num_ukiere_after_improve_discard);
                break;
            }
            // TODO handle upgrade case
        }
    }
    println!("summary:");
    println!(
        "{} total trials, {} reached tenpai in at most {} draws",
        num_trials, num_times_reached_tenpai, max_allowed_draws
    );
    let avg_turns_to_reach_tenpai =
        (total_num_turns_to_reach_tenpai as f64) / (num_times_reached_tenpai as f64);
    let avg_num_ukiere_at_tenpai =
        (total_num_ukiere_at_tenpai as f64) / (num_times_reached_tenpai as f64);
    println!(
        "average {:.2} turns to reach tenpai (among success)",
        avg_turns_to_reach_tenpai
    );
    println!(
        "average {:.2} ukiere tiles at tenpai (choosing discard that maximizes ukiere)",
        avg_num_ukiere_at_tenpai
    );
    println!(
        "count of number of trials that we reached tenpai: (starting from first draw)\n{:?}",
        count_turns_reached_tenpai
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_basic_analysis() {
        let hand_before_discard = shanten::tiles_to_count_array("345m11256p46778s6s");
        let starting_hand = shanten::remove_tile_id_from_count_array(
            hand_before_discard,
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
        );
        // visible tiles:
        // dora indicator: 6s
        // discard pools:
        // self   (east): 4z2z3z9m9p7z6m2p
        // shimo (south): 8p4z1z2z3p3p9s
        // toimen (west): 4z5z6z3z1s3z7z
        // kami  (north): 4z1z6z5z1s4m9m
        let visible_tiles = mahjong_tile::get_tile_ids_from_string(
            "6s4z2z3z9m9p7z6m2p8p4z1z2z3p3p9s4z5z6z3z1s3z7z4z1z6z5z1s4m9m",
        );
        run_basic_analysis(starting_hand, &visible_tiles, 100_000, 12);
    }

    #[test]
    fn test_copy_tile_count_array() {
        let mut tile_count_array = shanten::tiles_to_count_array("135m");
        let new_count_array = tile_count_array;
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();
        assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 0);
        assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);

        tile_count_array.0[usize::from(tile_id_4m)] += 1;
        assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 1);
        assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);
    }

    #[bench]
    fn bench_copy_tile_count_array(b: &mut Bencher) {
        // 12.34 ns/iter (+/- 0.06)
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();
        let mut tile_count_array = shanten::tiles_to_count_array("135m");
        b.iter(|| {
            tile_count_array.0[usize::from(tile_id_4m)] = 0;
            let new_count_array = tile_count_array;
            tile_count_array.0[usize::from(tile_id_4m)] = 1;
            assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 1);
            assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);
            tile_count_array
        });
    }

    #[test]
    fn test_manual_copy_tile_count_array() {
        let mut tile_count_array = shanten::tiles_to_count_array("135m");
        let mut new_count_array: MahjongTileCountArray = Default::default();
        for tile_id in 0..tile_count_array.0.len() {
            new_count_array.0[tile_id] = tile_count_array.0[tile_id];
        }
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();
        assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 0);
        assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);

        tile_count_array.0[usize::from(tile_id_4m)] += 1;
        assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 1);
        assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);
    }

    #[test]
    fn test_random_tile_distribution() {
        let mut visible_tile_ids = mahjong_tile::get_tile_ids_from_string(
            "6s4z2z3z9m9p7z6m2p8p4z1z2z3p3p9s4z5z6z3z1s3z7z4z1z6z5z1s4m9m",
        );
        // hand after discarding 2p
        let mut hand_tile_ids = mahjong_tile::get_tile_ids_from_string("345m1156p46778s6s");
        visible_tile_ids.append(&mut hand_tile_ids);

        let remaining_tile_count =
            remove_tile_ids_from_count_array(FOUR_OF_EACH_TILE_COUNT_ARRAY, &visible_tile_ids);
        println!("actual remaining tile count:\n{:?}", remaining_tile_count);
        let expected_remaining_tile_count: MahjongTileCountArray = MahjongTileCountArray([
            4, 4, 3, 2, 3, 3, 4, 4, 2, // manzu
            2, 3, 2, 4, 3, 3, 4, 3, 3, // pinzu
            2, 4, 4, 3, 4, 1, 2, 3, 3, // souzu
            2, 2, 1, 0, 2, 2, 2, // honor
        ]);
        let tile_id_4p = mahjong_tile::get_id_from_tile_text("4p").unwrap();
        let tile_id_7p = mahjong_tile::get_id_from_tile_text("7p").unwrap();
        let tile_id_5s = mahjong_tile::get_id_from_tile_text("5s").unwrap();
        let tile_id_8s = mahjong_tile::get_id_from_tile_text("8s").unwrap();
        assert_eq!(remaining_tile_count, expected_remaining_tile_count);
        assert_eq!(
            remaining_tile_count.0[usize::from(tile_id_4p)]
                + remaining_tile_count.0[usize::from(tile_id_7p)]
                + remaining_tile_count.0[usize::from(tile_id_5s)]
                + remaining_tile_count.0[usize::from(tile_id_8s)],
            15
        );

        let mut generated_tile_count_array = [0u32; 34]; // need u32 here to count instances
        let num_iters = 1_000_000;
        for _i in 0..num_iters {
            let generated_tile_id = generate_random_tile_id_rng(remaining_tile_count);
            generated_tile_count_array[usize::from(generated_tile_id)] += 1;
        }
        println!(
            "generated tile distribution: over {} iterations\n{:?}",
            num_iters, generated_tile_count_array
        );
        let mut num_ukiere_iters: u32 = 0;
        num_ukiere_iters += generated_tile_count_array[usize::from(tile_id_4p)];
        num_ukiere_iters += generated_tile_count_array[usize::from(tile_id_7p)];
        num_ukiere_iters += generated_tile_count_array[usize::from(tile_id_5s)];
        num_ukiere_iters += generated_tile_count_array[usize::from(tile_id_8s)];
        println!(
            "sampled percentage of drawing an ukiere tile = {:.2}%",
            (num_ukiere_iters as f64) / (num_iters as f64) * 100.0
        );
        println!(
            "expected percentage (15 / 93) = {:.2}%",
            15.0 / 93.0 * 100.0
        );
    }

    #[bench]
    fn bench_manual_copy_tile_count_array(b: &mut Bencher) {
        // 12.34 ns/iter (+/- 0.11)
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();
        let mut tile_count_array = shanten::tiles_to_count_array("135m");
        b.iter(|| {
            tile_count_array.0[usize::from(tile_id_4m)] = 0;
            let mut new_count_array: MahjongTileCountArray = Default::default();
            for tile_id in 0..tile_count_array.0.len() {
                new_count_array.0[tile_id] = tile_count_array.0[tile_id];
            }
            tile_count_array.0[usize::from(tile_id_4m)] = 1;
            assert_eq!(tile_count_array.0[usize::from(tile_id_4m)], 1);
            assert_eq!(new_count_array.0[usize::from(tile_id_4m)], 0);
            tile_count_array
        });
    }

    #[bench]
    fn bench_generate_random_tile_id_rng(b: &mut Bencher) {
        // without the ThreadRng passed in: 786.13 ns / iter (+/- 10.42)
        let remaining_tile_count = FOUR_OF_EACH_TILE_COUNT_ARRAY;
        b.iter(|| generate_random_tile_id_rng(remaining_tile_count));
    }

    #[bench]
    fn bench_generate_random_tile_id(b: &mut Bencher) {
        // with the ThreadRng passed in: 803.49 ns / iter (+/- 13.53)
        let remaining_tile_count = FOUR_OF_EACH_TILE_COUNT_ARRAY;
        let mut rng = rand::rng();
        b.iter(|| generate_random_tile_id(remaining_tile_count, &mut rng));
    }

    #[bench]
    fn bench_remove_tile_ids_from_count_array(b: &mut Bencher) {
        // 30.17 ns/iter (+/- 1.05)
        let remaining_tile_count = FOUR_OF_EACH_TILE_COUNT_ARRAY;
        let visible_tile_ids = mahjong_tile::get_tile_ids_from_string(
            "6s4z2z3z9m9p7z6m2p8p4z1z2z3p3p9s4z5z6z3z1s3z7z4z1z6z5z1s4m9m",
        );
        b.iter(|| remove_tile_ids_from_count_array(remaining_tile_count, &visible_tile_ids));
    }
}

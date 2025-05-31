use std::collections::HashMap;

pub mod mahjong_error;
// pub mod mahjong_game_state;
pub mod mahjong_hand;
pub mod mahjong_meld;
pub mod mahjong_tile;
pub mod shanten;
pub mod mini_mahjong;
pub mod state;
pub mod tile_grouping;
pub mod tiles;
pub mod yaku;

fn main() {
    // for serial in 0..tiles::NUM_TILES {
    //     let tile = tiles::Tile { serial };
    //     // print!("{} ", tile.to_string());
    //     print!("{} ", tile.to_human_string());
    //     if '(serial < 3 * 36 && serial % 9 == 8) || (serial >= 3 * 36 && (serial - 3 * 36) % 7 == 6)
    //     {
    //         println!("");
    //     }
    // }

    // let game_state = mini_mahjong::simulator::initialize_mini_game_state();

    let fixed_game_state = mini_mahjong::simulator::MiniGameState {
        hand_tiles: vec![
            mini_mahjong::mini_game::MiniTile { serial: 1 }, // 2p
            mini_mahjong::mini_game::MiniTile { serial: 10 }, // 2p
            mini_mahjong::mini_game::MiniTile { serial: 4 }, // 5p
            mini_mahjong::mini_game::MiniTile { serial: 6 }, // 7p
            mini_mahjong::mini_game::MiniTile { serial: 0 }, // 1p
        ],
        dead_tiles_by_rank: HashMap::from([(2, 2), (5, 1), (7, 1), (1, 1)]),
    };

    // how many turns does it take to get to a winning hand, on average?
    // to reduce variance, we can start with the same initial hand, but with different shuffle of the wall
    let num_trials = 1000000;
    let strategies: Vec<(&str, fn(&mini_mahjong::simulator::MiniGameState) -> usize)> = vec![
        // ("discard_random", mini_mahjong::strategy::discard_random),
        // (
        //     "discard_lowest",
        //     mini_mahjong::strategy::discard_lowest_rank,
        // ),
        // (
        //     "discard_highest",
        //     mini_mahjong::strategy::discard_highest_rank,
        // ),
        // ("discard_isolated", mini_mahjong::strategy::discard_isolated),
        // (
        //     "hardcoded_initial_wait",
        //     mini_mahjong::strategy::hardcoded_initial_wait,
        // ),
        ("hold_tenpai", mini_mahjong::strategy::hold_tenpai),
    ];
    for (strategy_name, discard_strategy) in strategies {
        println!("discard strategy: {:?}", strategy_name);
        let mut total_draws_to_win = 0;
        let mut total_draws_wins_only = 0;
        let mut total_wins = 0;
        for _i in 0..num_trials {
            let (draws_to_win, did_win) =
                mini_mahjong::simulator::play_game(&fixed_game_state, discard_strategy);
            total_draws_to_win += draws_to_win;
            if did_win {
                total_draws_wins_only += draws_to_win;
                total_wins += 1;
            }
        }
        let win_percentage = (total_wins as f32) * 100.0 / (num_trials as f32);
        let avg_draws = (total_draws_to_win as f32) / (num_trials as f32);

        println!(
            "initial hand: {}",
            mini_mahjong::simulator::display_hand(&fixed_game_state.hand_tiles)
        );
        println!("{num_trials} trials: win % = {win_percentage}, avg draws = {avg_draws}");
        if total_wins > 0 {
            let avg_draws_to_win = (total_draws_wins_only as f32) / (total_wins as f32);
            println!("avg draws (wins only) = {avg_draws_to_win}");
        }
    }
}

pub use crate::mahjong_tile;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiichiInfo {
    Riichi {
        is_ippatsu: bool,
        is_double_riichi: bool,
    },
    NoRiichi,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HandState {
    Open,
    Closed { riichi_info: RiichiInfo },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WinningTileSource {
    Discard {
        is_last_discard: bool, // winning on the last discard is the yaku "houtei raoyui"
    }, // TODO add player id that discarded?
    SelfDraw {
        is_first_draw: bool, // note: winning on first draw as dealer is the yakuman "tenhou" / "blessing of heaven". Winning on the first draw as non-dealer (with no prior tile calls by any player) is the yakuman "chiihou" / "blessing of earth"
        is_last_draw: bool,  // winning on the last draw is the yaku "haitei raoyue"
    },
    AfterKan, // winning on the replacement tile drawn after declaring a kan is the yaku "rinshan kaihou"
    RobbingKan, // winning by robbing a kan is the yaku "chankan"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WinningTileInfo {
    source: WinningTileSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HandInfo {
    hand_state: HandState,
    round_wind: mahjong_tile::MahjongWindOrder,
    seat_wind: mahjong_tile::MahjongWindOrder,
    round_number: u8, // 1-4 inclusive (represents how many different players have been dealer, including this hand)
    honba_counter: u16, // this number +1 is the "repeat"/renchan number e.g. east round, 2nd dealer, 1 honba -> East-2, 1st bonus round
    dora_tiles: Vec<u8>, // TODO should be Vec<mahjong_tile::MahjongTileId>,
}

/// note: maximum possible fu is 110 fu, so using a u8 to represent fu is okay
pub fn compute_han_and_fu(
    hand_tiles: Vec<u8>, // TODO should be Vec<mahjong_tile::MahjongTileId>,
    winning_tile: u8,    // TODO should be mahjong_tile::MahjongTileId,
    hand_info: HandInfo,
    winning_tile_info: WinningTileInfo,
) -> (u8, u8) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test::Bencher;

    #[test]
    fn jpml_sample_pro_test_hand_scoring_q1() {
        // TODO
        let hand = mahjong_tile::tiles_to_tile_ids("123m11222p23456s");
        let dora_tiles = mahjong_tile::tiles_to_tile_ids("1p");
        let win_tile = mahjong_tile::get_id_from_tile_text("1s").unwrap();

        // winning methods (ron vs tsumo)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::SelfDraw {
                is_last_draw: false,
            },
        };

        // situations (dealer in East-1 vs. south in East-1)
        let as_dealer = HandInfo {
            hand_state: HandState::Closed {
                riichi_info: RiichiInfo::Riichi {
                    is_ippatsu: false,
                    is_double_riichi: false,
                },
            },
            round_wind: mahjong_tile::MahjongWindOrder::East,
            seat_wind: mahjong_tile::MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = mahjong_tile::MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), tsumo.clone())
        );

        // Dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), ron.clone())
        );

        // Non-dealer Tsumo = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), tsumo.clone())
        );

        // Non-dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), ron.clone())
        );

        // let hand_info_options = vec![as_dealer, as_nondealer];
        // let winning_tile_info_options = vec![ron, tsumo];
        // for hand_info in hand_info_options {
        //     for winning_tile_info in winning_tile_info_options {
        //     }
        // }
    }
}

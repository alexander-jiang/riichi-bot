// use crate::tiles;

// #[derive(Copy, Clone)]
// pub enum WindDirection {
//     East = 1,
//     South,
//     West,
//     North,
// }

// impl WindDirection {
//     pub fn to_rank(&self) -> tiles::TileRank {
//         match self {
//             Self::East => tiles::TileRank::Honor(tiles::HonorTileRank::East),
//             Self::South => tiles::TileRank::Honor(tiles::HonorTileRank::South),
//             Self::West => tiles::TileRank::Honor(tiles::HonorTileRank::West),
//             Self::North => tiles::TileRank::Honor(tiles::HonorTileRank::North),
//         }
//     }
// }

// #[derive(Copy, Clone)]
// pub enum Dragon {
//     White = 5,
//     Green,
//     Red,
// }

// impl Dragon {
//     pub fn to_rank(&self) -> tiles::TileRank {
//         match self {
//             Self::White => tiles::TileRank::Honor(tiles::HonorTileRank::White),
//             Self::Green => tiles::TileRank::Honor(tiles::HonorTileRank::Green),
//             Self::Red => tiles::TileRank::Honor(tiles::HonorTileRank::Red),
//         }
//     }
// }

// #[derive(Copy, Clone)]
// pub enum WinningTileSource {
//     // i.e. ron
//     Discard,
//     // i.e. tsumo
//     SelfDraw,
//     // i.e. after the player calls a kan
//     DeadWall,
//     // i.e. winning off of another player's added kan (i.e. they had an open triplet and drew the fourth copy)
//     // - exception is if a player is tenpai for the suukantsu yaku (i.e. 4 quads), then they cannot ron off of a 5th added-quad tile
//     RobbingKan,
// }

// /// A new hand begins with a new set of initial tiles (haipai). Multiple hands make up a wind round, 
// /// and an entire game may consist of multiple wind rounds.
// /// Not to be confused with a player's hand, which is a set of tiles that belong to a specific player.
// pub struct HandState {
//     /// The round wind direction. used for yaku (yakuhai)
//     pub round_wind: WindDirection,
//     /// whether any player has made a call. if so, cancels chiihou, and the "all four players discard the same wind tile in first turn" abortive draw
//     pub any_calls_made: bool,
//     /// how many tiles are left in the wall (excluding the dead wall). used for ending a hand, as well as the haitei and houtei yaku (i.e. winning off of draw from last live wall tile, or winning off of last discarded tile)
//     pub tiles_remaining: u32,
//     /// The dora indicator tiles in the wall
//     pub dora_indicators: Vec<tiles::Tile>,
//     /// The number of riichi sticks that have been placed so far in this hand. used for scoring
//     pub riichi_sticks: u32,
//     /// The number of honba sticks for this hand. used for scoring
//     pub honba_sticks: u32,
// }

// pub struct PlayerState {
//     /// The ordered list of this player's discards. Includes tiles called by other players.
//     /// Used for furiten, as well as nagashi mangan
//     /// If empty, means there is a chance for tenhou / chiihou i.e. win on first draw
//     pub discards: Vec<tiles::Tile>,
//     /// The player's seat wind direction. used for yaku (yakuhai)
//     pub seat_wind: WindDirection,
//     /// Whether the player's hand is in riichi. used for scoring
//     pub in_riichi: bool,
//     /// Whether the player's hand is in double riichi. used for scoring
//     pub in_double_riichi: bool,
//     /// Whether the player's hand is eligible for ippatsu (i.e. in riichi or double riichi,
//     /// and no tile calls have been made and the player has not discarded a tile). used for scoring
//     pub in_ippatsu_turn: bool,
//     /// Whether any of this player's discards have been called by another player.
//     /// used for nagashi mangan (a special case yaku based on reaching exhaustive draw, having
//     /// only discarded honors & terminals, and none of those tiles were called by other players)
//     pub any_discards_called_by_others: bool,
//     /// Set to the player's winning tile source (if any). used for scoring, and certain yaku
//     pub winning_tile_source: Option<WinningTileSource>,
// }

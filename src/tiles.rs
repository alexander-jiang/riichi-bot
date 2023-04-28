// pub enum NumberTileType {
//     One   = 1,
//     Two   = 2,
//     Three = 3,
//     Four  = 4,
//     Five  = 5,
//     Six   = 6,
//     Seven = 7,
//     Eight = 8,
//     Nine  = 9,
// }
 
// // also represents the ordering for dora
// pub enum WindTileType {
//     East  = 1,
//     South = 2,
//     West  = 3,
//     North = 4,
// }

// // also represents the ordering for dora
// pub enum DragonTileType {
//     White = 1,
//     Green = 2,
//     Red   = 3,
// }

// pub enum TileSuit {
//     Man,
//     Pin,
//     Sou,
//     Wind,
//     Dragon,
// }

// union TileRank {
//     NumberTileType::One,
//     NumberTileType::Two,
//     NumberTileType::Three,
//     NumberTileType::Four,
//     NumberTileType::Five,
//     NumberTileType::Six,
//     NumberTileType::Seven,
//     NumberTileType::Eight,
//     NumberTileType::Nine,
//     WindTileType,
//     DragonTileType,
// }

// // TODO should we flatten this into a single enum with one value for each suit (man, pin, sou, wind, dragon)?
// pub struct Tile {
//     suit: TileSuit,
//     rank: TileRank,
// }

// impl Tile {
//     fn repr(&self) {
//         let rankChar: &str = match self.rank {
//             NumberTileType::One   => "1",
//             NumberTileType::Two   => "2",
//             NumberTileType::Three => "3",
//             NumberTileType::Four  => "4",
//             NumberTileType::Five  => "5",
//             NumberTileType::Six   => "6",
//             NumberTileType::Seven => "7",
//             NumberTileType::Eight => "8",
//             NumberTileType::Nine  => "9",
//             WindTileType::East    => "E",
//             WindTileType::South   => "S",
//             WindTileType::West    => "W",
//             WindTileType::North   => "N",
//             DragonTileType::White => "W",
//             DragonTileType::Green => "G",
//             DragonTileType::Red   => "R",
//         };
//         let suitChar: &str = match self.suit {
//             TileSuit::Man    => "m",
//             TileSuit::Pin    => "p",
//             TileSuit::Sou    => "s",
//             TileSuit::Wind   => "w",
//             TileSuit::Dragon => "d",
//         };
//         rankChar.clone() + suitChar
//     }
// }

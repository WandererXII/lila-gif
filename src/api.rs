use arrayvec::ArrayString;
use serde::{de, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use shogi::{Position, Square, Move, Color, Piece};
use std::fmt;


#[derive(Deserialize, PartialEq, Eq, Copy, Clone)]
pub enum Orientation {
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "white")]
    White,
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation::Black
    }
}

impl Orientation {
    pub fn fold<T>(self, black: T, white: T) -> T {
        match self {
            Orientation::Black => black,
            Orientation::White => white,
        }
    }

    pub fn eq_color(self, color: Color) -> bool {
        match self {
            Orientation::Black => color == Color::Black,
            Orientation::White => color == Color::White,
        }
    }

    pub fn x(self, square: Square) -> usize {
        self.fold(8 - usize::from(square.file()), usize::from(square.file()))
    }

    pub fn y(self, square: Square) -> usize {
        self.fold(usize::from(square.rank()), 8 - usize::from(square.rank()))
    }

    pub fn hand_y(self, piece: Piece) -> usize {
        if self.eq_color(piece.color) { 9 - piece.piece_type as usize } else { piece.piece_type as usize - 1 }
    }
}

pub type PlayerName = ArrayString<100>; // length limited to prevent dos

pub type Comment = ArrayString<255>; // strict length limit for gif comments

#[derive(Copy, Clone)]
pub enum CheckSquare {
    No,
    Yes,
    Square(Square),
}

impl Default for CheckSquare {
    fn default() -> CheckSquare {
        CheckSquare::No
    }
}

impl<'de> Deserialize<'de> for CheckSquare {
    fn deserialize<D>(deseralizer: D) -> Result<CheckSquare, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct CheckSquareVisitor;

        impl<'de> de::Visitor<'de> for CheckSquareVisitor {
            type Value = CheckSquare;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("square name or bool")
            }

            fn visit_str<E>(self, name: &str) -> Result<CheckSquare, E>
            where
                E: de::Error,
            {
                if name == "1" || name == "yes" || name == "true" {
                    Ok(CheckSquare::Yes)
                } else if name == "0" || name == "no" || name == "false" {
                    Ok(CheckSquare::No)
                } else {
                    match Square::from_sfen(name) {
                        Some(sq) => Ok(CheckSquare::Square(sq)),
                        None => Err(de::Error::custom("invalid square name"))
                    }
                }
            }

            fn visit_bool<E>(self, yes: bool) -> Result<CheckSquare, E>
            where
                E: de::Error,
            {
                Ok(match yes {
                    true => CheckSquare::Yes,
                    false => CheckSquare::No,
                })
            }
        }

        deseralizer.deserialize_any(CheckSquareVisitor)
    }
}

impl CheckSquare {
    pub fn to_square(self) -> Option<Square> {
        match self {
            CheckSquare::No => None,
            CheckSquare::Yes => Square::from_sfen("e5"),
            CheckSquare::Square(sq) => Some(sq),
        }
    }
}

#[serde_as]
#[derive(Deserialize)]
pub struct RequestParams {
    pub black: Option<PlayerName>,
    pub white: Option<PlayerName>,
    pub comment: Option<Comment>,
    #[serde_as(as = "DisplayFromStr")]
    pub sfen: String,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default, rename = "lastMove")]
    pub last_move: Option<Move>,
    #[serde(default)]
    pub check: CheckSquare,
    #[serde(default)]
    pub orientation: Orientation,
}

#[derive(Deserialize)]
pub struct RequestBody {
    pub black: Option<PlayerName>,
    pub white: Option<PlayerName>,
    pub comment: Option<Comment>,
    pub frames: Vec<RequestFrame>,
    #[serde(default)]
    pub orientation: Orientation,
    #[serde(default)]
    pub delay: u16,
}

#[serde_as]
#[derive(Deserialize, Default)]
pub struct RequestFrame {
    #[serde_as(as = "DisplayFromStr")]
    pub sfen: String,
    #[serde(default)]
    pub delay: Option<u16>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(default, rename = "lastMove")]
    pub last_move: Option<Move>,
    #[serde(default)]
    pub check: CheckSquare,
}

impl RequestBody {
    pub fn example() -> RequestBody {
        let moves = "7g7f 4c4d 8i7g 3c3d 7g6e 2b3c 6e5c+ 3d3e 5c6c 4d4e 8h3c+ 2a3c B*6f 8b4b 6f3c+ 7c7d N*4d B*1e 6c5b 6a5b 4d5b+ 4a5b 3c1a N*3b 1g1f 1e2d G*1e 4b4a 1e2d 2c2d L*6f 3a4b B*9e 5b6b 6f6c+ 4a1a 5g5f B*5d 2h7h 5d6c 7f7e 9c9d 9e7g 9d9e 7g1a+ 6c5b 7e7d 6b6c R*5e L*5c 5e4e N*3c 4e3e G*4d 3e8e 7a7b 1a1b 1c1d 1b2c 5a4a 2c1d 4b3a 1d1e 4d4e 5i4h 5c5f P*5g P*5a 5g5f 4e5f L*4f P*4b P*5g 5f6g 4f4b+ 3a4b 7h7f L*1a 1e2f 8a9c 8e6e P*6d 6e6g 3b4d 2f4d 5b2e 7f4f 6c7d 4d3c P*3a 3c2d 2e4c G*4d 7d7c 4d4c 4b4c 4f4c+ G*4b 2d4b";

        let mut frames = Vec::with_capacity(46 * 2 + 1);
        frames.push(RequestFrame {sfen: "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(), ..Default::default()});

        let mut pos = Position::new();
        pos.set_sfen("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1").unwrap();

        for move_str in moves.split(" ") {
            if move_str.trim().is_empty() || move_str.ends_with(".") {
                continue;
            }
            let m = Move::from_sfen(move_str).unwrap();
            pos.make_move(m).unwrap();
            let sfen = pos.to_sfen();

            frames.push(RequestFrame {
                sfen: sfen,
                check: if pos.in_check(pos.side_to_move().flip()) { CheckSquare::Yes } else { CheckSquare::No },
                last_move: Some(m),
                delay: None,
            })
        }

        frames.last_mut().unwrap().delay = Some(500);

        RequestBody {
            comment: Some(Comment::from("Nowhere").unwrap()),
            black: Some(PlayerName::from("Sente").unwrap()),
            white: Some(PlayerName::from("Gote").unwrap()),
            orientation: Orientation::Black,
            delay: 75,
            frames: frames,
        }
    }
}

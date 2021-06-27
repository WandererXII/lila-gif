use gift::block::{ColorTableConfig, GlobalColorTable};
use ndarray::{s, Array2, ArrayView2};
use rusttype::Font;
use shogi::{Piece, PieceType, Color};

use crate::api::Orientation;

const SQUARE: usize = 90;
const COLOR_WIDTH: usize = 90 * 2 / 3;

pub struct SpriteKey {
    pub piece: Option<Piece>,
    pub orientation: Orientation,
    pub highlight: bool,
    pub check: bool,
}

// WIP only, make it nicer
impl SpriteKey {
    fn x(&self) -> usize {
        let mx = match self.piece {
            Some(piece) if piece.piece_type == PieceType::King => 0,
            Some(piece) if piece.piece_type == PieceType::Rook => 0,
            Some(piece) if piece.piece_type == PieceType::Bishop => 0,
            Some(piece) if piece.piece_type == PieceType::Gold => 0,
            Some(piece) if piece.piece_type == PieceType::Silver => 1,
            Some(piece) if piece.piece_type == PieceType::Knight => 1,
            Some(piece) if piece.piece_type == PieceType::Lance => 1,
            Some(piece) if piece.piece_type == PieceType::Pawn => 1,
            Some(piece) if piece.piece_type == PieceType::ProRook => 2,
            Some(piece) if piece.piece_type == PieceType::ProBishop => 2,
            Some(piece) if piece.piece_type == PieceType::ProSilver => 2,
            Some(piece) if piece.piece_type == PieceType::ProKnight => 3,
            Some(piece) if piece.piece_type == PieceType::ProLance => 3,
            Some(piece) if piece.piece_type == PieceType::ProPawn => 3,
            Some(_) => 3,
            None => 2
        };
        2 * mx + if self.highlight && self.piece.is_some() { 1 } else { 0 }
    }

    fn y(&self) -> usize {
        let my = match self.piece {
            Some(piece) if piece.piece_type == PieceType::King => 3,
            Some(piece) if piece.piece_type == PieceType::Rook => 2,
            Some(piece) if piece.piece_type == PieceType::Bishop => 1,
            Some(piece) if piece.piece_type == PieceType::Gold => 0,
            Some(piece) if piece.piece_type == PieceType::Silver => 3,
            Some(piece) if piece.piece_type == PieceType::Knight => 2,
            Some(piece) if piece.piece_type == PieceType::Lance => 1,
            Some(piece) if piece.piece_type == PieceType::Pawn => 0,
            Some(piece) if piece.piece_type == PieceType::ProRook => 3,
            Some(piece) if piece.piece_type == PieceType::ProBishop => 2,
            Some(piece) if piece.piece_type == PieceType::ProSilver => 1,
            Some(piece) if piece.piece_type == PieceType::ProKnight => 3,
            Some(piece) if piece.piece_type == PieceType::ProLance => 2,
            Some(piece) if piece.piece_type == PieceType::ProPawn => 1,
            Some(_) => 2,
            None => 0
        };
        let mc = match self.piece {
            Some(piece) if
                (piece.color == Color::White && self.orientation == Orientation::Black ||
                    piece.color == Color::Black && self.orientation == Orientation::White) => 1,
            Some(_) => 0,
            None if self.highlight => 0,
            None => 1
        };
        2 * my + mc
    }
}

pub struct Theme {
    color_table_config: ColorTableConfig,
    global_color_table: GlobalColorTable,
    sprite: Array2<u8>,
    font: Font<'static>,
}

impl Theme {
    pub fn new() -> Theme {
        let sprite_data = include_bytes!("../theme/sprite_new2.gif") as &[u8];
        let mut decoder = gift::Decoder::new(std::io::Cursor::new(sprite_data)).into_frames();
        let preamble = decoder.preamble().expect("decode preamble").expect("preamble");
        let frame = decoder.next().expect("frame").expect("decode frame");
        let sprite = Array2::from_shape_vec((SQUARE * 9, SQUARE * 9), frame.image_data.data().to_owned()).expect("from shape");

        let font_data = include_bytes!("../theme/NotoSans-Regular.ttf") as &[u8];
        let font = Font::try_from_bytes(font_data).expect("parse font");

        Theme {
            color_table_config: preamble.logical_screen_desc.color_table_config(),
            global_color_table: preamble.global_color_table.expect("color table present"),
            sprite,
            font,
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn color_table_config(&self) -> ColorTableConfig {
        self.color_table_config
    }

    pub fn global_color_table(&self) -> &GlobalColorTable {
        &self.global_color_table
    }

    pub fn bar_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5)]
    }

    pub fn text_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5 + COLOR_WIDTH)]
    }

    pub fn gold_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5 + COLOR_WIDTH * 2)]
    }

    pub fn bot_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5 + COLOR_WIDTH * 3)]
    }

    pub fn med_text_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5 + COLOR_WIDTH * 4)]
    }

    pub fn transparent_color(&self) -> u8 {
        self.sprite[(0, SQUARE * 5 + COLOR_WIDTH * 5)]
    }

    pub fn square(&self) -> usize {
        SQUARE
    }

    pub fn width(&self) -> usize {
        self.square() * 9
    }

    pub fn bar_height(&self) -> usize {
        60
    }

    pub fn height(&self, bars: bool) -> usize {
        if bars {
            self.width() + 2 * self.bar_height()
        } else {
            self.width()
        }
    }

    pub fn sprite(&self, key: SpriteKey) -> ArrayView2<u8> {
        //println!("{:?}", key.piece);
        let y = key.y() % 9;
        let x = key.x();
        //println!("{:?}", y);
        self.sprite.slice(s!((SQUARE * y)..(SQUARE + SQUARE * y), (SQUARE * x)..(SQUARE + SQUARE * x)))
    }
}

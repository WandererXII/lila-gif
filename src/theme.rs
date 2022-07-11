use gift::block::{ColorTableConfig, GlobalColorTable};
use ndarray::{s, Array2, ArrayView2};
use rusttype::Font;
use shogi::{Color, Piece, PieceType};

use crate::api::Orientation;

const SCALE: usize = 7;
const SQUARE_WIDTH: usize = 11 * SCALE;
const SQUARE_HEIGHT: usize = 12 * SCALE;
const CIRCLE: usize = 5;

pub struct SpriteHandKey {
    pub piece: Piece,
    pub orientation: Orientation,
    pub number: u8,
}

impl SpriteHandKey {
    fn x(&self) -> usize {
        8 + if self.orientation.eq_color(self.piece.color) {
            0
        } else {
            1
        } + if self.number > 0 { 2 } else { 0 }
    }

    fn y(&self) -> usize {
        9 - self.piece.piece_type as usize
    }
}

pub struct SpriteKey {
    pub piece: Option<Piece>,
    pub orientation: Orientation,
    pub highlight: bool,
    pub check: bool,
}

impl SpriteKey {
    fn x(&self) -> usize {
        let mx = match self.piece {
            Some(piece) if self.check && piece.piece_type == PieceType::King => 3,
            Some(piece) if piece.piece_type == PieceType::King && piece.color == Color::Black => 4,
            Some(piece) => (piece.piece_type as usize) / 4,
            None => 5,
        };
        2 * mx + if self.highlight { 1 } else { 0 }
    }

    fn y(&self) -> usize {
        match self.piece {
            Some(piece) if self.check && piece.piece_type == PieceType::King && piece.color == Color::Black => {
                5 + self.orientation.fold(1, 0)
            }
            Some(piece) if self.check && piece.piece_type == PieceType::King && piece.color == Color::White => {
                7 + self.orientation.fold(0, 1)
            }
            Some(piece) if piece.piece_type == PieceType::King && piece.color == Color::Black => {
                0 + self.orientation.fold(1, 0)
            }
            Some(piece) => {
                2 * ((piece.piece_type as usize) % 4) + if self.orientation.eq_color(piece.color) { 1 } else { 0 } + 1
            }
            None => 0,
        }
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
        let sprite_data = include_bytes!("../theme/sprite.gif") as &[u8];
        let mut decoder = gift::Decoder::new(std::io::Cursor::new(sprite_data)).into_frames();
        let preamble = decoder.preamble().expect("decode preamble").expect("preamble");
        let frame = decoder.next().expect("frame").expect("decode frame");
        let sprite = Array2::from_shape_vec(
            (SQUARE_HEIGHT * 9, SQUARE_WIDTH * 12),
            frame.image_data.data().to_owned(),
        )
        .expect("from shape");

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
        self.sprite[(0, 0)]
    }

    pub fn text_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH)]
    }

    pub fn gold_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 2)]
    }

    pub fn bot_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 3)]
    }

    pub fn med_text_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 4)]
    }

    pub fn hand_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 5)]
    }

    pub fn white_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 6)]
    }

    pub fn transparent_color(&self) -> u8 {
        self.sprite[(0, SQUARE_WIDTH * 7)]
    }

    pub fn circle_color(&self) -> u8 {
        self.sprite[(SQUARE_HEIGHT + SQUARE_HEIGHT / 2, SQUARE_WIDTH * 10 + SQUARE_WIDTH / 2)]
    }

    pub fn square_width(&self) -> usize {
        SQUARE_WIDTH
    }

    pub fn square_height(&self) -> usize {
        SQUARE_HEIGHT
    }

    pub fn circle(&self) -> usize {
        CIRCLE
    }

    pub fn hand_width(&self) -> usize {
        self.square_width() + self.square_width() / 2
    }

    pub fn hand_offset(&self) -> usize {
        self.square_width() / 3
    }

    pub fn board_width(&self) -> usize {
        self.square_width() * 9
    }

    pub fn board_height(&self) -> usize {
        self.square_height() * 9
    }

    pub fn width(&self) -> usize {
        self.square_width() * 12
    }

    pub fn bar_height(&self) -> usize {
        60
    }

    pub fn height(&self, bars: bool) -> usize {
        if bars {
            self.square_height() * 9 + 2 * self.bar_height()
        } else {
            self.square_height() * 9
        }
    }

    pub fn circle_sprite(&self, bottom: bool, right: bool) -> ArrayView2<u8> {
        let circle_center_top = SQUARE_HEIGHT + SQUARE_HEIGHT / 2 - if bottom { self.circle() } else { 0 };
        let circle_center_left = SQUARE_WIDTH * 10 + SQUARE_WIDTH / 2 - if right { self.circle() } else { 0 };
        self.sprite.slice(s!(
            (circle_center_top)..(circle_center_top + self.circle()),
            (circle_center_left)..(circle_center_left + self.circle())
        ))
    }

    pub fn sprite(&self, key: SpriteKey) -> ArrayView2<u8> {
        let y = key.y() % 9;
        let x = key.x() % 12;
        self.sprite.slice(s!(
            (SQUARE_HEIGHT * y)..(SQUARE_HEIGHT + SQUARE_HEIGHT * y),
            (SQUARE_WIDTH * x)..(SQUARE_WIDTH + SQUARE_WIDTH * x)
        ))
    }

    pub fn hand_sprite(&self, key: SpriteHandKey) -> ArrayView2<u8> {
        let y = key.y() % 9;
        let x = key.x() % 12;
        self.sprite.slice(s!(
            (SQUARE_HEIGHT * y + 1)..(SQUARE_HEIGHT + SQUARE_HEIGHT * y),
            (SQUARE_WIDTH * x + 1)..(SQUARE_WIDTH + SQUARE_WIDTH * x)
        ))
    }
}

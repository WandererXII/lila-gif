use shogi::Move;
use bytes::{Bytes, BytesMut, BufMut};
use gift::{block, Encoder};
use ndarray::{s, ArrayViewMut2};
use shogi::{Bitboard, Position, PieceType, Color};
use shogi::bitboard::Factory;
use std::iter::FusedIterator;
use std::vec;
use rusttype::Scale;

use crate::api::{Comment, Orientation, PlayerName, RequestBody, RequestParams};
use crate::theme::{SpriteKey, Theme};

enum RenderState {
    Preamble,
    Frame(RenderFrame),
    Complete,
}

struct PlayerBars {
    black: PlayerName,
    white: PlayerName,
}

impl PlayerBars {
    fn from(black: Option<PlayerName>, white: Option<PlayerName>) -> Option<PlayerBars> {
        if black.is_some() || white.is_some() {
            Some(PlayerBars {
                black: black.unwrap_or_default(),
                white: white.unwrap_or_default(),
            })
        } else {
            None
        }
    }
}

#[derive(Default)]
#[derive(Debug)]
struct RenderFrame {
    sfen: String,
    checked: Bitboard,
    highlighted: Bitboard,
    delay: Option<u16>,
}

impl RenderFrame {
    fn diff(&self, prev: &RenderFrame) -> Bitboard {
        Factory::init();
        let mut prev_pos = Position::new();
        prev_pos.set_sfen(&prev.sfen).unwrap();
        let mut cur_pos = Position::new();
        cur_pos.set_sfen(&self.sfen).unwrap();
        (prev.checked ^ self.checked) |
        (prev.highlighted ^ self.highlighted) |
        (prev_pos.player_bb(Color::Black) ^ cur_pos.player_bb(Color::Black)) |
        (prev_pos.piece_bb(PieceType::Pawn) ^ cur_pos.piece_bb(PieceType::Pawn)) |
        (prev_pos.piece_bb(PieceType::Lance) ^ cur_pos.piece_bb(PieceType::Lance)) |
        (prev_pos.piece_bb(PieceType::Knight) ^ cur_pos.piece_bb(PieceType::Knight)) |
        (prev_pos.piece_bb(PieceType::Silver) ^ cur_pos.piece_bb(PieceType::Silver)) |
        (prev_pos.piece_bb(PieceType::Gold) ^ cur_pos.piece_bb(PieceType::Gold)) |
        (prev_pos.piece_bb(PieceType::Bishop) ^ cur_pos.piece_bb(PieceType::Bishop)) |
        (prev_pos.piece_bb(PieceType::Rook) ^ cur_pos.piece_bb(PieceType::Rook)) |
        (prev_pos.piece_bb(PieceType::King) ^ cur_pos.piece_bb(PieceType::King)) |
        (prev_pos.piece_bb(PieceType::ProPawn) ^ cur_pos.piece_bb(PieceType::ProPawn)) |
        (prev_pos.piece_bb(PieceType::ProLance) ^ cur_pos.piece_bb(PieceType::ProLance)) |
        (prev_pos.piece_bb(PieceType::ProKnight) ^ cur_pos.piece_bb(PieceType::ProKnight)) |
        (prev_pos.piece_bb(PieceType::ProSilver) ^ cur_pos.piece_bb(PieceType::ProSilver)) |
        (prev_pos.piece_bb(PieceType::ProBishop) ^ cur_pos.piece_bb(PieceType::ProBishop)) |
        (prev_pos.piece_bb(PieceType::ProRook) ^ cur_pos.piece_bb(PieceType::ProRook))
    }
}

pub struct Render {
    theme: &'static Theme,
    state: RenderState,
    buffer: Vec<u8>,
    comment: Option<Comment>,
    bars: Option<PlayerBars>,
    orientation: Orientation,
    frames: vec::IntoIter<RenderFrame>,
    kork: bool,
}

impl Render {
    pub fn new_image(theme: &'static Theme, params: RequestParams) -> Render {
        let bars = params.black.is_some() || params.white.is_some();
        Render {
            theme,
            buffer: vec![0; theme.height(bars) * theme.width()],
            state: RenderState::Preamble,
            comment: params.comment,
            bars: PlayerBars::from(params.black, params.white),
            orientation: params.orientation,
            frames: vec![RenderFrame {
                sfen: params.sfen,
                highlighted: highlight_uci(params.last_move),
                checked: params.check.to_square().map(|sq| Bitboard::from_square(sq)).unwrap_or(Bitboard::empty()),
                delay: None,
            }].into_iter(),
            kork: false,
        }
    }

    pub fn new_animation(theme: &'static Theme, params: RequestBody) -> Render {
        let bars = params.black.is_some() || params.white.is_some();
        let default_delay = params.delay;

        Render {
            theme,
            buffer: vec![0; theme.height(bars) * theme.width()],
            state: RenderState::Preamble,
            comment: params.comment,
            bars: PlayerBars::from(params.black, params.white),
            orientation: params.orientation,
            frames: params.frames.into_iter().map(|frame| RenderFrame {
                sfen: frame.sfen,
                highlighted: highlight_uci(frame.last_move),
                checked: frame.check.to_square().map(|sq| Bitboard::from_square(sq)).unwrap_or(Bitboard::empty()),
                delay: Some(frame.delay.unwrap_or(default_delay)),
            }).collect::<Vec<_>>().into_iter(),
            kork: true,
        }
    }
}

impl Iterator for Render {
    type Item = Bytes;

    fn next(&mut self) -> Option<Bytes> {
        let mut output = BytesMut::new().writer();
        match self.state {
            RenderState::Preamble => {
                let mut blocks = Encoder::new(&mut output).into_block_enc();

                blocks.encode(block::Header::default()).expect("enc header");
                
                blocks.encode(
                    block::LogicalScreenDesc::default()
                        .with_screen_height(self.theme.height(self.bars.is_some()) as u16)
                        .with_screen_width(self.theme.width() as u16)
                        .with_color_table_config(self.theme.color_table_config())
                ).expect("enc logical screen desc");

                blocks.encode(
                    self.theme.global_color_table().clone()
                ).expect("enc global color table");

                blocks.encode(
                    block::Application::with_loop_count(0)
                ).expect("enc application");

                let comment = self.comment.as_ref().map_or("https://github.com/niklasf/lila-gif".as_bytes(), |c| c.as_bytes());
                if !comment.is_empty() {
                    let mut comments = block::Comment::default();
                    comments.add_comment(comment);
                    blocks.encode(comments).expect("enc comment");
                }

                let mut view = ArrayViewMut2::from_shape(
                    (self.theme.height(self.bars.is_some()), self.theme.width()),
                    &mut self.buffer
                ).expect("shape");

                let mut board_view = if let Some(ref bars) = self.bars {
                    render_bar(
                        view.slice_mut(s!(..self.theme.bar_height(), ..)),
                        self.theme,
                        self.orientation.fold(&bars.white, &bars.black));

                    render_bar(
                        view.slice_mut(s!((self.theme.bar_height() + self.theme.width()).., ..)),
                        self.theme,
                        self.orientation.fold(&bars.black, &bars.white));

                    view.slice_mut(s!(self.theme.bar_height()..(self.theme.bar_height() + self.theme.width()), ..))
                } else {
                    view
                };

                let frame = self.frames.next().unwrap_or_default();

                if let Some(delay) = frame.delay {
                    let mut ctrl = block::GraphicControl::default();
                    ctrl.set_delay_time_cs(delay);
                    blocks.encode(ctrl).expect("enc graphic control");
                }

                render_diff(
                    board_view.as_slice_mut().expect("continguous"),
                    self.theme,
                    self.orientation,
                    None,
                    &frame);

                blocks.encode(
                    block::ImageDesc::default()
                        .with_height(self.theme.height(self.bars.is_some()) as u16)
                        .with_width(self.theme.width() as u16)
                ).expect("enc image desc");

                let mut image_data = block::ImageData::new(self.buffer.len());
                image_data.data_mut().extend_from_slice(&self.buffer);
                blocks.encode(image_data).expect("enc image data");

                self.state = RenderState::Frame(frame);
            }
            RenderState::Frame(ref prev) => {
                let mut blocks = Encoder::new(&mut output).into_block_enc();

                if let Some(frame) = self.frames.next() {
                    let mut ctrl = block::GraphicControl::default();
                    ctrl.set_disposal_method(block::DisposalMethod::Keep);
                    ctrl.set_transparent_color_idx(self.theme.transparent_color());
                    if let Some(delay) = frame.delay {
                        ctrl.set_delay_time_cs(delay);
                    }
                    blocks.encode(ctrl).expect("enc graphic control");

                    let ((left, y), (w, h)) = render_diff(
                        &mut self.buffer,
                        self.theme,
                        self.orientation,
                        Some(&prev),
                        &frame);

                    let top = y + if self.bars.is_some() { self.theme.bar_height() } else { 0 };

                    blocks.encode(
                        block::ImageDesc::default()
                            .with_left(left as u16)
                            .with_top(top as u16)
                            .with_height(h as u16)
                            .with_width(w as u16)
                    ).expect("enc image desc");

                    let mut image_data = block::ImageData::new(w * h);
                    image_data.data_mut().extend_from_slice(&self.buffer[..(w * h)]);
                    blocks.encode(image_data).expect("enc image data");

                    self.state = RenderState::Frame(frame);
                } else {
                    // Add a black frame at the end, to work around twitter
                    // cutting off the last frame.
                    if self.kork {
                        let mut ctrl = block::GraphicControl::default();
                        ctrl.set_disposal_method(block::DisposalMethod::Keep);
                        ctrl.set_transparent_color_idx(self.theme.transparent_color());
                        ctrl.set_delay_time_cs(1);
                        blocks.encode(ctrl).expect("enc graphic control");

                        let height = self.theme.height(self.bars.is_some());
                        let width = self.theme.width();
                        blocks.encode(
                            block::ImageDesc::default()
                                .with_left(0)
                                .with_top(0)
                                .with_height(height as u16)
                                .with_width(width as u16)
                        ).expect("enc image desc");

                        let mut image_data = block::ImageData::new(height * width);
                        image_data.data_mut().resize(height * width, self.theme.bar_color());
                        blocks.encode(image_data).expect("enc image data");
                    }

                    blocks.encode(block::Trailer::default()).expect("enc trailer");
                    self.state = RenderState::Complete;
                }
            }
            RenderState::Complete => return None,
        }
        Some(output.into_inner().freeze())
    }
}

impl FusedIterator for Render { }

fn render_diff(buffer: &mut [u8], theme: &Theme, orientation: Orientation, prev: Option<&RenderFrame>, frame: &RenderFrame) -> ((usize, usize), (usize, usize)) {
    Factory::init();
    let diff = prev.map_or(Factory::all(), |p| p.diff(frame));

    let x_min = diff.into_iter().map(|sq| orientation.x(sq)).min().unwrap_or(0);
    let y_min = diff.into_iter().map(|sq| orientation.y(sq)).min().unwrap_or(0);
    let x_max = diff.into_iter().map(|sq| orientation.x(sq)).max().unwrap_or(0) + 1;
    let y_max = diff.into_iter().map(|sq| orientation.y(sq)).max().unwrap_or(0) + 1;

    let width = (x_max - x_min) * theme.square();
    let height = (y_max - y_min) * theme.square();

    let mut view = ArrayViewMut2::from_shape((height, width), buffer).expect("shape");

    if prev.is_some() {
        view.fill(theme.transparent_color());
    }

    let mut pos = Position::new();
    pos.set_sfen(&frame.sfen).unwrap();
    for sq in diff {
        let key = SpriteKey {
            piece: *pos.piece_at(sq),
            orientation: orientation,
            highlight: frame.highlighted.is_occupied(sq),
            check: frame.checked.is_occupied(sq),
        };
        let left = (orientation.x(sq) - x_min) * theme.square();
        let top = (orientation.y(sq) - y_min) * theme.square();

        view.slice_mut(
            s!(top..(top + theme.square()), left..(left + theme.square()))
        ).assign(&theme.sprite(key));
    }

    ((theme.square() * x_min, theme.square() * y_min), (width, height))
}

fn render_bar(mut view: ArrayViewMut2<u8>, theme: &Theme, player_name: &str) {
    view.fill(theme.bar_color());

    let mut text_color = theme.text_color();
    if player_name.starts_with("BOT ") {
        text_color = theme.bot_color();
    } else {
        for title in &["GM ", "BOT "] {
            if player_name.starts_with(title) {
                text_color = theme.gold_color();
                break;
            }
        }
    }

    let height = 40.0;
    let padding = 10.0;
    let scale = Scale {
        x: height,
        y: height,
    };

    let v_metrics = theme.font().v_metrics(scale);
    let glyphs = theme.font().layout(player_name, scale, rusttype::point(padding, padding + v_metrics.ascent));

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|left, top, intensity| {
                let left = left as i32 + bb.min.x;
                let top = top as i32 + bb.min.y;
                if 0 <= left && left < theme.width() as i32 && 0 <= top && top < theme.bar_height() as i32 {
                    // Poor man's anti-aliasing.
                    if intensity < 0.01 {
                        return;
                    } else if intensity < 0.5 && text_color == theme.text_color() {
                        view[(top as usize, left as usize)] = theme.med_text_color();
                    } else {
                        view[(top as usize, left as usize)] = text_color;
                    }
                }
            });
        } else {
            text_color = theme.text_color();
        }
    }
}

fn highlight_uci(m: Option<Move>) -> Bitboard {
    match m {
        Some(Move::Normal { from, to, .. }) => Bitboard::from_square(from) | Bitboard::from_square(to),
        Some(Move::Drop { to, .. }) => Bitboard::from_square(to),
        _ => Bitboard::empty(),
    }
}

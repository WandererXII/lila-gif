use bytes::{BufMut, Bytes, BytesMut};
use gift::{block, Encoder};
use ndarray::{s, ArrayViewMut2};
use rusttype::Scale;
use shogi::bitboard::Factory;
use shogi::Move;
use shogi::{Bitboard, Color, Piece, PieceType, Position, Square};
use std::iter::FusedIterator;
use std::vec;

use crate::api::{Comment, Orientation, PlayerName, RequestBody, RequestParams};
use crate::theme::{SpriteHandKey, SpriteKey, Theme};

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

#[derive(Default, Debug)]
struct RenderFrame {
    sfen: Position,
    checked: Bitboard,
    highlighted: Bitboard,
    delay: Option<u16>,
}

impl RenderFrame {
    fn diff(&self, prev: &RenderFrame) -> Bitboard {
        (&prev.checked ^ &self.checked)
            | (&prev.highlighted ^ &self.highlighted)
            | (prev.sfen.player_bb(Color::Black) ^ self.sfen.player_bb(Color::Black))
            | (prev.sfen.piece_bb(PieceType::Pawn) ^ self.sfen.piece_bb(PieceType::Pawn))
            | (prev.sfen.piece_bb(PieceType::Lance) ^ self.sfen.piece_bb(PieceType::Lance))
            | (prev.sfen.piece_bb(PieceType::Knight) ^ self.sfen.piece_bb(PieceType::Knight))
            | (prev.sfen.piece_bb(PieceType::Silver) ^ self.sfen.piece_bb(PieceType::Silver))
            | (prev.sfen.piece_bb(PieceType::Gold) ^ self.sfen.piece_bb(PieceType::Gold))
            | (prev.sfen.piece_bb(PieceType::Bishop) ^ self.sfen.piece_bb(PieceType::Bishop))
            | (prev.sfen.piece_bb(PieceType::Rook) ^ self.sfen.piece_bb(PieceType::Rook))
            | (prev.sfen.piece_bb(PieceType::King) ^ self.sfen.piece_bb(PieceType::King))
            | (prev.sfen.piece_bb(PieceType::ProPawn) ^ self.sfen.piece_bb(PieceType::ProPawn))
            | (prev.sfen.piece_bb(PieceType::ProLance) ^ self.sfen.piece_bb(PieceType::ProLance))
            | (prev.sfen.piece_bb(PieceType::ProKnight) ^ self.sfen.piece_bb(PieceType::ProKnight))
            | (prev.sfen.piece_bb(PieceType::ProSilver) ^ self.sfen.piece_bb(PieceType::ProSilver))
            | (prev.sfen.piece_bb(PieceType::ProBishop) ^ self.sfen.piece_bb(PieceType::ProBishop))
            | (prev.sfen.piece_bb(PieceType::ProRook) ^ self.sfen.piece_bb(PieceType::ProRook))
    }

    fn hand_diff(&self, prev: &RenderFrame) -> Vec<Piece> {
        let mut t: Vec<Piece> = Vec::new();

        for pt in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
            for c in Color::iter() {
                let piece = Piece {
                    color: c,
                    piece_type: pt,
                };
                if prev.sfen.hand(piece) != self.sfen.hand(piece) {
                    t.push(piece);
                }
            }
        }
        t
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
                highlighted: highlight_uci(params.last_move),
                checked: params
                    .check
                    .to_square(params.sfen.find_king(params.sfen.side_to_move()))
                    .map(|sq| Bitboard::from_square(sq))
                    .unwrap_or(Bitboard::empty()),
                sfen: params.sfen,
                delay: None,
            }]
            .into_iter(),
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
            frames: params
                .frames
                .into_iter()
                .map(|frame| RenderFrame {
                    highlighted: highlight_uci(frame.last_move),
                    checked: frame
                        .check
                        .to_square(frame.sfen.find_king(frame.sfen.side_to_move()))
                        .map(|sq| Bitboard::from_square(sq))
                        .unwrap_or(Bitboard::empty()),
                    sfen: frame.sfen,
                    delay: Some(frame.delay.unwrap_or(default_delay)),
                })
                .collect::<Vec<_>>()
                .into_iter(),
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

                blocks
                    .encode(
                        block::LogicalScreenDesc::default()
                            .with_screen_height(self.theme.height(self.bars.is_some()) as u16)
                            .with_screen_width(self.theme.width() as u16)
                            .with_color_table_config(self.theme.color_table_config()),
                    )
                    .expect("enc logical screen desc");

                blocks
                    .encode(self.theme.global_color_table().clone())
                    .expect("enc global color table");

                blocks
                    .encode(block::Application::with_loop_count(0))
                    .expect("enc application");

                let comment = self
                    .comment
                    .as_ref()
                    .map_or("https://github.com/WandererXII/lishogi-gif".as_bytes(), |c| {
                        c.as_bytes()
                    });
                if !comment.is_empty() {
                    let mut comments = block::Comment::default();
                    comments.add_comment(comment);
                    blocks.encode(comments).expect("enc comment");
                }

                let mut view = ArrayViewMut2::from_shape(
                    (self.theme.height(self.bars.is_some()), self.theme.width()),
                    &mut self.buffer,
                )
                .expect("shape");

                let mut board_view = if let Some(ref bars) = self.bars {
                    render_bar(
                        view.slice_mut(s!(..self.theme.bar_height(), ..)),
                        self.theme,
                        self.orientation.fold(&bars.white, &bars.black),
                    );
                    render_bar(
                        view.slice_mut(s!((self.theme.bar_height() + self.theme.board_width()).., ..)),
                        self.theme,
                        self.orientation.fold(&bars.black, &bars.white),
                    );
                    render_hand(
                        view.slice_mut(s!(
                            self.theme.bar_height()..(self.theme.bar_height() + self.theme.board_width()),
                            ..self.theme.hand_width()
                        )),
                        self.theme,
                    );
                    render_hand(
                        view.slice_mut(s!(
                            self.theme.bar_height()..(self.theme.bar_height() + self.theme.board_width()),
                            (self.theme.hand_width() + self.theme.board_width())..
                        )), // self.theme.bar_height()..(self.theme.bar_height() + self.theme.board_width())
                        self.theme,
                    );
                    view.slice_mut(s!(
                        self.theme.bar_height()..(self.theme.bar_height() + self.theme.board_width()),
                        ..
                    ))
                } else {
                    render_hand(view.slice_mut(s!(.., ..self.theme.hand_width())), self.theme);
                    render_hand(
                        view.slice_mut(s!(.., (self.theme.hand_width() + self.theme.board_width())..)),
                        self.theme,
                    );
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
                    &frame,
                );

                blocks
                    .encode(
                        block::ImageDesc::default()
                            .with_height(self.theme.height(self.bars.is_some()) as u16)
                            .with_width(self.theme.width() as u16),
                    )
                    .expect("enc image desc");

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

                    let ((left, y), (w, h)) =
                        render_diff(&mut self.buffer, self.theme, self.orientation, Some(&prev), &frame);

                    let top = y + if self.bars.is_some() {
                        self.theme.bar_height()
                    } else {
                        0
                    };

                    blocks
                        .encode(
                            block::ImageDesc::default()
                                .with_left(left as u16)
                                .with_top(top as u16)
                                .with_height(h as u16)
                                .with_width(w as u16),
                        )
                        .expect("enc image desc");

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
                        blocks
                            .encode(
                                block::ImageDesc::default()
                                    .with_left(0)
                                    .with_top(0)
                                    .with_height(height as u16)
                                    .with_width(width as u16),
                            )
                            .expect("enc image desc");

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

impl FusedIterator for Render {}

fn render_diff(
    buffer: &mut [u8],
    theme: &Theme,
    orientation: Orientation,
    prev: Option<&RenderFrame>,
    frame: &RenderFrame,
) -> ((usize, usize), (usize, usize)) {
    let diff = prev.map_or(Factory::all(), |p| p.diff(frame));

    let hand_diff: Vec<Piece> = prev.map_or(
        Color::iter()
            .flat_map(|c| {
                PieceType::iter()
                    .filter(|pt| pt.is_hand_piece())
                    .map(|pt| Piece {
                        piece_type: pt,
                        color: c,
                    })
                    .collect::<Vec<Piece>>()
            })
            .collect(),
        |p| p.hand_diff(frame),
    );

    let hand_left = hand_diff.iter().any(|p| !orientation.eq_color(p.color));
    let hand_right = hand_diff.iter().any(|p| orientation.eq_color(p.color));

    let x_min = if hand_left {
        0
    } else {
        diff.into_iter()
            .map(|sq| orientation.x(sq) * theme.square())
            .min()
            .unwrap_or(0)
            + theme.hand_width()
    };
    let x_max = if hand_right {
        theme.width()
    } else {
        diff.into_iter()
            .map(|sq| orientation.x(sq) * theme.square())
            .max()
            .unwrap_or(0)
            + theme.hand_width()
            + theme.square()
    };

    let y_min = std::cmp::min(
        hand_diff
            .iter()
            .map(|p| orientation.hand_y(*p) * theme.square())
            .min()
            .unwrap_or(9),
        diff.into_iter()
            .map(|sq| orientation.y(sq) * theme.square())
            .min()
            .unwrap_or(0),
    );
    let y_max = std::cmp::max(
        hand_diff
            .iter()
            .map(|p| orientation.hand_y(*p) * theme.square())
            .max()
            .unwrap_or(0)
            + theme.square(),
        diff.into_iter()
            .map(|sq| orientation.y(sq) * theme.square())
            .max()
            .unwrap_or(0)
            + theme.square(),
    );

    let width = x_max - x_min;
    let height = y_max - y_min;

    let mut view = ArrayViewMut2::from_shape((height, width), buffer).expect("shape");

    if prev.is_some() {
        view.fill(theme.transparent_color());
    }

    let center_squares = Factory::all()
        .filter(|sq| {
            (sq.rank() == 2 || sq.rank() == 3 || sq.rank() == 5 || sq.rank() == 6)
                && (sq.file() == 2 || sq.file() == 3 || sq.file() == 5 || sq.file() == 6)
        })
        .collect::<Vec<Square>>();

    for sq in diff {
        let key = SpriteKey {
            piece: *frame.sfen.piece_at(sq),
            orientation: orientation,
            highlight: frame.highlighted.is_occupied(sq),
            check: frame.checked.is_occupied(sq),
        };
        let left = theme.hand_width() + orientation.x(sq) * theme.square() - x_min;
        let top = orientation.y(sq) * theme.square() - y_min;

        view.slice_mut(s!(top..(top + theme.square()), left..(left + theme.square())))
            .assign(&theme.sprite(key));

        if center_squares.contains(&sq) {
            let top_circle = if orientation.y(sq) == 2 || orientation.y(sq) == 5 {
                top + theme.square() - theme.circle()
            } else {
                top
            };
            let left_circle = if orientation.x(sq) == 2 || orientation.x(sq) == 5 {
                left + theme.square() - theme.circle()
            } else {
                left
            };
            view.slice_mut(s!(
                (top_circle)..(top_circle + theme.circle()),
                (left_circle)..(left_circle + theme.circle())
            ))
            .zip_mut_with(&theme.circle_sprite(top != top_circle, left != left_circle), |x, y| {
                if *y != theme.transparent_color() as u8 {
                    *x = y.clone()
                }
            });
        }
    }

    for p in hand_diff {
        let nb = std::cmp::min(frame.sfen.hand(p), 99);

        let key = SpriteHandKey {
            piece: p,
            orientation: orientation,
            number: nb,
        };
        let left = if orientation.eq_color(p.color) {
            width - theme.square() - theme.hand_offset() / 2
        } else {
            theme.hand_offset() / 2
        };
        let top = orientation.hand_y(p) * theme.square() - y_min;

        // +1 to cut of border - todo fix sprite
        view.slice_mut(s!(
            (top + 1)..(top + theme.square()),
            (left + 1)..(left + theme.square())
        ))
        .assign(&theme.hand_sprite(key));

        if nb > 0 {
            let mut text_color = theme.white_color();
            let font_size = 30.0;
            let x_offset = 69;
            let y_offset = 60;
            let scale = Scale {
                x: font_size,
                y: font_size,
            };
            let g_text = nb.to_string();
            let g_center = (g_text.len() - 1) as f32 * 6.0;
            let v_metrics = theme.font().v_metrics(scale);
            let glyphs = theme.font().layout(
                g_text.as_str(),
                scale,
                rusttype::point(
                    (x_offset + left) as f32 - g_center,
                    (y_offset + top) as f32 + v_metrics.ascent,
                ),
            );

            for g in glyphs {
                if let Some(bb) = g.pixel_bounding_box() {
                    g.draw(|left, top, intensity| {
                        let left = left as i32 + bb.min.x;
                        let top = top as i32 + bb.min.y;
                        if 0 <= left && left < width as i32 && 0 <= top && top < height as i32 {
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
    }

    ((x_min, y_min), (width, height))
}

fn render_hand(mut view: ArrayViewMut2<u8>, theme: &Theme) {
    view.fill(theme.hand_color());
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
    let scale = Scale { x: height, y: height };

    let v_metrics = theme.font().v_metrics(scale);
    let glyphs = theme.font().layout(
        player_name,
        scale,
        rusttype::point(padding + theme.hand_width() as f32, padding + v_metrics.ascent),
    );

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

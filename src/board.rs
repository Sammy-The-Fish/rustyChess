use std::{collections::HashMap, hash::Hash};

use crate::{
    PieceColor, Pieces, Pos, SIZE,
    pieces::{self, Move, MoveType},
};

pub enum GameResult {
    BlackWin,
    WhiteWin,
    Draw { reason: DrawReason },
}

pub enum DrawReason {
    Stalemate,
    Agreed,
    ThreefoldRepeat,
    FivefoldRepeat,
    InsufficientMaterial,
    FiftyMoves,
    SeventyFiveMoves,
}

pub struct Board {
    board: [[Pieces; SIZE]; SIZE],
    moves: i32,
    last_capture: i32,
    history: HashMap<BoardState, i32>,
}

#[derive(PartialEq, Eq, Hash)]
struct BoardState {
    state: [[i32; SIZE]; SIZE],
    is_white_turn: bool,
}

impl Eq for PieceColor {}

impl Board {
    pub fn new() -> Board {
        let mut board: [[Pieces; SIZE]; SIZE] =
            std::array::from_fn(|_| std::array::from_fn(|_| Pieces::Empty));

        // set up black back rank
        board[0][0] = Pieces::Rook {
            side: (PieceColor::White),
            is_first_move: true,
        };
        board[0][1] = Pieces::Knight {
            side: (PieceColor::White),
        };
        board[0][2] = Pieces::Bishop {
            side: (PieceColor::White),
        };
        board[0][3] = Pieces::Queen {
            side: (PieceColor::White),
        };
        board[0][4] = Pieces::King {
            side: (PieceColor::White),
            is_first_move: true,
        };
        board[0][5] = Pieces::Bishop {
            side: (PieceColor::White),
        };
        board[0][6] = Pieces::Knight {
            side: (PieceColor::White),
        };
        board[0][7] = Pieces::Rook {
            side: (PieceColor::White),
            is_first_move: true,
        };

        // set up white pawns
        for i in 0..SIZE {
            board[1][i] = Pieces::Pawn {
                side: (PieceColor::White),
                is_first_move: true,
                has_just_jumped: false,
            };
        }

        // set up white back rank
        board[SIZE - 1][0] = Pieces::Rook {
            side: (PieceColor::Black),
            is_first_move: true,
        };
        board[SIZE - 1][1] = Pieces::Knight {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][2] = Pieces::Bishop {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][3] = Pieces::Queen {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][4] = Pieces::King {
            side: (PieceColor::Black),
            is_first_move: true,
        };
        board[SIZE - 1][5] = Pieces::Bishop {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][6] = Pieces::Knight {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][7] = Pieces::Rook {
            side: (PieceColor::Black),
            is_first_move: true,
        };

        // set up black pawns
        for i in 0..SIZE {
            board[SIZE - 2][i] = Pieces::Pawn {
                side: (PieceColor::Black),
                is_first_move: true,
                has_just_jumped: false,
            };
        }

        // board modifications to make testing easier

        Board {
            board,
            moves: 0,
            last_capture: 0,
            history: HashMap::new(),
        }
    }
    // might implement get_king better later
    pub fn get_king_position(&self, side: &PieceColor) -> Pos {
        for i in 0..SIZE {
            for j in 0..SIZE {
                let piece = self.get_piece_at_pos((i, j));
                match piece {
                    Pieces::King {
                        side: piece_side, ..
                    } => {
                        if piece_side == side {
                            return (i, j);
                        }
                    }
                    _ => (),
                }
            }
        }
        // if no king on board, panic
        panic!();
    }
    pub fn get_all_pieces(&self, side: &PieceColor) -> Vec<Pos> {
        let mut result = Vec::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                let piece = self.get_piece_at_pos((i, j));
                if let Some(color) = piece.color() {
                    if color == side {
                        result.push((i, j));
                    }
                }
            }
        }
        result
    }

    pub fn check_win_cons(&self, side: &PieceColor) -> Option<GameResult> {
        // check for check
        let mut checked = false;
        let king = self.get_king_position(side);
        if pieces::is_pos_checked(king, self, side) {
            checked = true
        }

        let mut valid_moves = false;
        // check for legal moves
        for piece in self.get_all_pieces(side) {
            let piece_moves = self.get_piece_at_pos(piece).find_moves(self, piece);

            if piece_moves.len() > 0 {
                valid_moves = true;
                break;
            }
        }

        // check win cons
        // checkmate
        if checked && !valid_moves {
            match side {
                PieceColor::Black => return Some(GameResult::WhiteWin),
                PieceColor::White => return Some(GameResult::BlackWin),
            }
        }

        // stalemate
        if !valid_moves {
            return Some(GameResult::Draw {
                reason: DrawReason::Stalemate,
            });
        }

        self.check_forced_draw_cons()
    }

    pub fn check_claimed_draw_cons(&self) -> Option<GameResult> {
        // threefold repetition
        let Some(max) = self.history.values().max() else {
            return None;
        };

        if *max >= 3 {
            return Some(GameResult::Draw {
                reason: DrawReason::ThreefoldRepeat,
            });
        }

        // 50 moves
        if self.moves - self.last_capture >= 100 {
            return Some(GameResult::Draw {
                reason: DrawReason::FiftyMoves,
            });
        }

        None
    }

    pub fn check_forced_draw_cons(&self) -> Option<GameResult> {
        // fivefold repetition
        let Some(max) = self.history.values().max() else {
            return None;
        };

        if *max >= 5 {
            return Some(GameResult::Draw {
                reason: DrawReason::FivefoldRepeat,
            });
        }

        // 75 moves
        if self.moves - self.last_capture >= 100 {
            return Some(GameResult::Draw {
                reason: DrawReason::SeventyFiveMoves,
            });
        }

        if self.insufficient_material() {
            return Some(GameResult::Draw {
                reason: DrawReason::InsufficientMaterial,
            });
        }
        None
    }

    pub fn insufficient_material(&self) -> bool {
        let mut pawns = 0;
        let mut rooks = 0;
        let mut knights = 0;
        let mut bishops = 0;
        let mut queens = 0;

        let mut pieces = self.get_all_pieces(&PieceColor::White);
        pieces.extend(self.get_all_pieces(&PieceColor::Black));

        for piece in pieces {
            match self.get_piece_at_pos(piece) {
                Pieces::Pawn { .. } => pawns += 1,
                Pieces::Rook { .. } => rooks += 1,
                Pieces::Knight { .. } => knights += 1,
                Pieces::Bishop { .. } => bishops += 1,
                Pieces::Queen { .. } => queens += 1,
                _ => (),
            }
        }

        if !(pawns == 0 && rooks == 0 && queens == 0) {
            return false;
        }

        // just kings
        if knights == 0 && bishops == 0 {
            return true;
        }

        // a king and a bishop
        if bishops == 1 && knights == 0 {
            return true;
        }

        // a king and a knight
        if knights == 1 && bishops == 0 {
            return true;
        }

        // TODO: need to implement more nuanced cases e.g. K + B v K + B
        false
    }

    pub fn is_checked(&self, side: &PieceColor) -> bool {
        let king = self.get_king_position(side);
        if pieces::is_pos_checked(king, &self, side) {
            return true;
        }
        false
    }

    pub fn get_piece_at_pos(&self, pos: (usize, usize)) -> &Pieces {
        &self.board[pos.0][pos.1]
    }

    pub fn get_mut_piece_at_pos(&mut self, pos: (usize, usize)) -> &mut Pieces {
        &mut self.board[pos.0][pos.1]
    }

    pub fn move_piece(&mut self, piece_move: Move) {
        let from = piece_move.from;
        let to = piece_move.to;

        let from_piece = self.get_mut_piece_at_pos(from);
        from_piece.on_move(from, to);

        self.moves += 1;

        let destination_empty = matches!(self.get_piece_at_pos(to), Pieces::Empty);

        if destination_empty {
            self.last_capture = self.moves;
        }

        let from_piece: &Pieces = self.get_piece_at_pos(from);
        let to_piece = self.get_piece_at_pos(to);
        let from_colour = from_piece.color().unwrap();
        let is_white = *from_colour == PieceColor::White;

        match piece_move.move_type {
            MoveType::Normal => {
                if let Some(to_color) = to_piece.color() {
                    if from_colour == to_color {
                        panic!()
                    }
                }

                let from_piece = std::mem::replace(&mut self.board[from.0][from.1], Pieces::Empty);

                self.board[to.0][to.1] = from_piece;
            }
            MoveType::Castle => {
                let direction: isize = to.1 as isize - from.1 as isize;

                // check which direction to castle in
                let is_kingside_castle = match direction {
                    2 => true,
                    -2 => false,
                    _ => {
                        panic!()
                    }
                };
                let rook_pos = if is_kingside_castle {
                    (to.0, 7)
                } else {
                    (to.0, 0)
                };

                let rook_target = if is_kingside_castle {
                    (to.0, 5)
                } else {
                    (to.0, 3)
                };

                // swap pieces
                let rook =
                    std::mem::replace(&mut self.board[rook_pos.0][rook_pos.1], Pieces::Empty);
                let from_piece = std::mem::replace(&mut self.board[from.0][from.1], Pieces::Empty);
                self.board[to.0][to.1] = from_piece;
                self.board[rook_target.0][rook_target.1] = rook;
            }
            MoveType::EnPassant => {
                println!("moving piece to {:?}", to);
                let pawn_pos = match from_colour {
                    PieceColor::Black => (to.0 + 1, to.1),
                    PieceColor::White => (to.0 - 1, to.1),
                };

                println!("clearing pawn at: {:?}", pawn_pos);
                // clear taken pawn
                self.board[pawn_pos.0][pawn_pos.1] = Pieces::Empty;

                // move pawn normally
                let from_piece = std::mem::replace(&mut self.board[from.0][from.1], Pieces::Empty);

                self.board[to.0][to.1] = from_piece;
            }
            MoveType::Promotion { piece } => {
                if let Some(to_color) = to_piece.color() {
                    if from_colour == to_color {
                        panic!()
                    }
                }

                let new_color = match from_colour {
                    PieceColor::Black => PieceColor::Black,
                    PieceColor::White => PieceColor::White,
                };
                let _ = std::mem::replace(&mut self.board[from.0][from.1], Pieces::Empty);

                let promoted_piece = match piece {
                    pieces::Promotion::Queen => Pieces::Queen { side: new_color },
                    pieces::Promotion::Rook => Pieces::Rook {
                        side: new_color,
                        is_first_move: false,
                    },
                    pieces::Promotion::Knight => Pieces::Knight { side: new_color },
                    pieces::Promotion::Bishop => Pieces::Bishop { side: new_color },
                };

                self.board[to.0][to.1] = promoted_piece;
            }
        }
        // add new position to history
        let mut state = [[0; SIZE]; SIZE];

        for i in 0..SIZE {
            for j in 0..SIZE {
                state[i][j] = hash_piece(self.get_piece_at_pos((i as usize, j as usize)))
            }
        }
        let board_state = BoardState {
            state,
            is_white_turn: is_white,
        };

        self.history
            .entry(board_state)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}

pub fn add_positions(pos1: Pos, pos2: (isize, isize)) -> Option<Pos> {
    let result = ((pos1.0 as isize + pos2.0), (pos1.1 as isize + pos2.1));

    if result.0 < 0 || result.0 >= SIZE as isize {
        return None;
    }
    if result.1 < 0 || result.1 >= SIZE as isize {
        return None;
    }

    Some((result.0 as usize, result.1 as usize))
}

// provides a unique number for each piece
fn hash_piece(piece: &Pieces) -> i32 {
    const EMPTY: i32 = 0;
    const PAWN: i32 = 1;
    const ROOK: i32 = 3;
    const KNIGHT: i32 = 5;
    const BISHOP: i32 = 7;
    const QUEEN: i32 = 9;
    const KING: i32 = 11;

    const BLACK_MOD: i32 = 1;
    const FIRST_MOVE_MOD: i32 = 100;
    const JUST_JUMPED_MOD: i32 = 1000;

    match piece {
        Pieces::Empty => EMPTY,
        Pieces::Pawn {
            side,
            is_first_move: _,
            has_just_jumped,
        } => {
            let mut result = PAWN;
            if *side == PieceColor::Black {
                result += BLACK_MOD;
            }
            if *has_just_jumped {
                return result;
            } else {
                return result + JUST_JUMPED_MOD;
            };
        }
        Pieces::Rook {
            side,
            is_first_move,
        } => {
            let mut result = ROOK;
            if *side == PieceColor::Black {
                result += BLACK_MOD;
            }
            if *is_first_move {
                return result;
            } else {
                return result + FIRST_MOVE_MOD;
            };
        }
        Pieces::Knight { side } => {
            if *side == PieceColor::Black {
                return KNIGHT + BLACK_MOD;
            } else {
                KNIGHT
            }
        }
        Pieces::Bishop { side } => {
            if *side == PieceColor::Black {
                return BISHOP + BLACK_MOD;
            } else {
                BISHOP
            }
        }
        Pieces::Queen { side } => {
            if *side == PieceColor::Black {
                return QUEEN + BLACK_MOD;
            } else {
                QUEEN
            }
        }
        Pieces::King {
            side,
            is_first_move,
        } => {
            let mut result = KING;
            if *side == PieceColor::Black {
                result += BLACK_MOD;
            }
            if *is_first_move {
                return result;
            } else {
                return result + FIRST_MOVE_MOD;
            };
        }
    }
}

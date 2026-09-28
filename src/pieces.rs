use std::{collections::HashMap, iter::Map, ops::Index, result, vec};

use crate::{
    Pos, SIZE,
    board::{self, Board},
};

#[derive(Debug)]
pub enum Pieces {
    Empty,
    Pawn {
        side: PieceColor,
        is_first_move: bool,
    },
    Rook {
        side: PieceColor,
    },
    Knight {
        side: PieceColor,
    },
    Bishop {
        side: PieceColor,
    },
    Queen {
        side: PieceColor,
    },
    King {
        side: PieceColor,
    },
}

impl PartialEq for Pieces {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pawn { .. }, Self::Pawn { .. }) => true,
            (Self::Rook { .. }, Self::Rook { .. }) => true,
            (Self::Knight { .. }, Self::Knight { .. }) => true,
            (Self::Bishop { .. }, Self::Bishop { .. }) => true,
            (Self::Queen { .. }, Self::Queen { .. }) => true,
            (Self::King { .. }, Self::King { .. }) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Pieces {
    pub fn symbol(&self) -> char {
        match self {
            Pieces::Empty => ' ',
            Pieces::Pawn {
                side: _,
                is_first_move,
            } => 'P',
            Pieces::Rook { side: _ } => 'R',
            Pieces::Knight { side: _ } => 'N',
            Pieces::Bishop { side: _ } => 'B',
            Pieces::Queen { side: _ } => 'Q',
            Pieces::King { side: _ } => 'K',
        }
    }

    pub fn color(&self) -> Option<&PieceColor> {
        match self {
            Pieces::Empty => None,
            Pieces::Pawn {
                side: color,
                is_first_move,
            } => Some(color),
            Pieces::Rook { side: color } => Some(color),
            Pieces::Knight { side: color } => Some(color),
            Pieces::Bishop { side: color } => Some(color),
            Pieces::Queen { side: color } => Some(color),
            Pieces::King { side: color } => Some(color),
        }
    }

    pub fn find_moves(&self, board: &board::Board, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut potential_result = Vec::new();
        match self {
            Pieces::Empty => (),
            Pieces::Pawn {
                side,
                is_first_move,
            } => {
                // do pawns move up or down
                let direction = {
                    if *side == PieceColor::White {
                        (1, 0)
                    } else {
                        (-1, 0)
                    }
                };

                // check in front
                if let Some(target_x) = pos.0.checked_add_signed(direction.0) {
                    if target_x < 8 {
                        let target = board.get_piece_at_pos((target_x, pos.1));
                        if target.color() == None {
                            potential_result.push((target_x, pos.1));
                        }
                    }
                };

                // check 2 in front for fist move
                if *is_first_move {
                    if let Some(target_x) = pos.0.checked_add_signed(direction.0 * 2) {
                        if target_x < 8 {
                            let target = board.get_piece_at_pos((target_x, pos.1));
                            if target.color() == None {
                                potential_result.push((target_x, pos.1));
                            }
                        }
                    };
                }

                // check diagonals for attacks
                let diagonals = [(direction.0, 1), (direction.0, -1)];
                for diagonal in diagonals {
                    // ensures targets cannot be < 0
                    let Some(target_x) = pos.0.checked_add_signed(diagonal.0) else {
                        continue;
                    };
                    let Some(target_y) = pos.1.checked_add_signed(diagonal.1) else {
                        continue;
                    };

                    // ensures targets cannot be > 8
                    if target_x >= SIZE || target_y >= SIZE {
                        continue;
                    }

                    let target = board.get_piece_at_pos((target_x, target_y));
                    if let Some(color) = target.color() {
                        if color != side {
                            potential_result.push((target_x, target_y));
                        }
                    }
                }

                // TODO en passant
            }
            Pieces::Rook { side } => {
                let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];

                potential_result = self.scan_in_directions(&directions, pos, board, side);
            }
            Pieces::Knight { side } => {
                // check all possible knight moves
                // check large jump on x axis and large jump on y axis
                for mut jump in [(2, 1), (1, 2)] {
                    // let mut jump = jump;
                    for i in 0..4 {
                        // iterate through all permutation of positive and negative
                        if i % 2 == 0 {
                            jump.0 *= -1;
                        } else {
                            jump.1 *= -1;
                        }

                        // ensures targets cannot be < 0
                        let Some(target_x) = pos.0.checked_add_signed(jump.0) else {
                            continue;
                        };
                        let Some(target_y) = pos.1.checked_add_signed(jump.1) else {
                            continue;
                        };

                        // ensures targets cannot be > 8
                        if target_x >= SIZE || target_y >= SIZE {
                            continue;
                        }

                        let target = board.get_piece_at_pos((target_x, target_y));
                        if let Some(color) = target.color() {
                            if color != side {
                                potential_result.push((target_x, target_y));
                            }
                        } else {
                            potential_result.push((target_x, target_y));
                        }
                    }
                }
            }
            Pieces::Bishop { side } => {
                let directions = [(1, 1), (-1, 1), (1, -1), (-1, -1)];

                potential_result = self.scan_in_directions(&directions, pos, board, side);
            }
            Pieces::Queen { side } => {
                let directions = [
                    (1, 0),
                    (0, 1),
                    (-1, 0),
                    (0, -1),
                    (1, 1),
                    (-1, 1),
                    (1, -1),
                    (-1, -1),
                ];

                potential_result = self.scan_in_directions(&directions, pos, board, side);
            }
            Pieces::King { side } => {
                let moves: Vec<(isize, isize)> = (-1..=1)
                    .flat_map(|x| (-1..=1).map(move |y| (x, y)))
                    .filter(|&(x, y)| (x, y) != (0, 0))
                    .collect();

                for piece_move in moves {
                    let Some(square) = board::add_positions(pos, piece_move) else {
                        continue;
                    };

                    let target = board.get_piece_at_pos(square);

                    match target {
                        Pieces::Empty => {
                            if !is_pos_checked(square, board, side) {
                                potential_result.push(square);
                            }
                        }
                        _ => (),
                    }
                    if let Some(color) = target.color() {
                        if color != side {
                            potential_result.push(square);
                        }
                    }
                }

                // TODO: Caslting
            }
        }

        // for each possible move modify the board and check if the king is in check
        let mut result = Vec::new();
        for piece_move in potential_result {
            let mut modification: HashMap<Pos, &Pieces> = HashMap::new();
            let piece = board.get_piece_at_pos(pos);
            let color = self.color().unwrap();

            // add new place to the hashmap
            modification.insert(piece_move, piece);
            // replace old pos with empty
            modification.insert(pos, &Pieces::Empty);

            println!("checking for checks with hasmap: {:?}", modification);
            if !is_pos_potentially_checked(board.get_king_position(color), board, color, modification) {
                result.push(piece_move);
            }
        }
        result
    }

    fn scan_in_directions(
        &self,
        directions: &[(isize, isize)],
        pos: (usize, usize),
        board: &Board,
        side: &PieceColor,
    ) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for dir in directions {
            let mut counter = 0;
            // loop until we reach edge of board or a piece
            'scan: loop {
                counter += 1;
                let piece_move = (dir.0 * counter, dir.1 * counter);

                // stop if index goes beneath 0
                let Some(target_x) = pos.0.checked_add_signed(piece_move.0) else {
                    break 'scan;
                };
                let Some(target_y) = pos.1.checked_add_signed(piece_move.1) else {
                    break 'scan;
                };

                // stop if goes above size
                if target_x >= SIZE || target_y >= SIZE {
                    break 'scan;
                }

                let target = board.get_piece_at_pos((target_x, target_y));
                if let Some(color) = target.color() {
                    if color != side {
                        result.push((target_x, target_y));
                    }
                    break 'scan;
                } else {
                    result.push((target_x, target_y));
                }
            }
        }
        result
    }
}

#[derive(Debug, PartialEq)]
pub enum PieceColor {
    Black,
    White,
}

impl PieceColor {
    pub fn other(&self) -> PieceColor {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

fn is_pos_potentially_checked(
    pos: Pos,
    board: &Board,
    side: &PieceColor,
    modifications: HashMap<Pos, &Pieces>,
) -> bool {

    // TODO: fix issue where taking piece into check is allowed

    // check for pawns
    let direction = { if *side == PieceColor::White { -1 } else { 1 } };

    let checks = [(direction, 1), (direction, -1)];

    for check in checks {
        if let Some(square) = board::add_positions(pos, check) {
            let piece = match modifications.get(&square) {
                Some(piece) => piece,
                None => &board.get_piece_at_pos(square),
            };

            match piece {
                Pieces::Pawn {
                    side: piece_side, ..
                } => {
                    if piece_side != side {
                        return true;
                    }
                }
                _ => (),
            }
        }
    }

    // check for knights
    let checks = [
        (1, 2),
        (-1, 2),
        (1, -2),
        (-1, -2),
        (2, 1),
        (-2, 1),
        (2, -1),
        (-2, -1),
    ];

    for check in checks {
        if let Some(square) = board::add_positions(pos, check) {
            let piece = match modifications.get(&square) {
                Some(piece) => piece,
                None => &board.get_piece_at_pos(square),
            };

            match piece {
                Pieces::Knight { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                }
                _ => (),
            }
        }
    }

    // check for Rooks/Queens
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for dir in dirs {
        let mut counter = 0;
        // loop until we reach edge of board or a piece
        'scan: loop {
            counter += 1;
            let piece_move = (dir.0 * counter, dir.1 * counter);

            // stop if index goes beneath 0
            let Some(target_x) = pos.0.checked_add_signed(piece_move.0) else {
                break 'scan;
            };
            let Some(target_y) = pos.1.checked_add_signed(piece_move.1) else {
                break 'scan;
            };

            // stop if goes above size
            if target_x >= SIZE || target_y >= SIZE {
                break 'scan;
            }

            let target = match modifications.get(&(target_x, target_y)) {
                Some(piece) => piece,
                None => &board.get_piece_at_pos((target_x, target_y)),
            };

            println!("scanning for queens have found {:?}, at {:?}", target, (target_x, target_y));
            match target {
                Pieces::Queen { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                    break;

                }
                Pieces::Rook { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::King { side: piece_side } => {
                    if counter == 1 && piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::Empty => {
                    continue;
                }
                _ => {
                    break 'scan;
                },
            }
        }
    }

    // check for Bishops/Queens/Kings
    let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    for dir in dirs {
        let mut counter = 0;
        // loop until we reach edge of board or a piece
        'scan: loop {
            counter += 1;
            let piece_move = (dir.0 * counter, dir.1 * counter);

            // stop if index goes beneath 0
            let Some(target_x) = pos.0.checked_add_signed(piece_move.0) else {
                break 'scan;
            };
            let Some(target_y) = pos.1.checked_add_signed(piece_move.1) else {
                break 'scan;
            };

            // stop if goes above size
            if target_x >= SIZE || target_y >= SIZE {
                break 'scan;
            }

            let target = match modifications.get(&(target_x, target_y)) {
                Some(piece) => piece,
                None => &board.get_piece_at_pos((target_x, target_y)),
            };
            match target {
                Pieces::Queen { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::Bishop { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::King { side: piece_side } => {
                    if counter == 1 && piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::Empty => {
                    continue;
                }
                _ => {
                    break 'scan;
                },
            }
        }
    }

    println!("this has returned false");
    false
}

pub fn is_pos_checked(pos: Pos, board: &Board, side: &PieceColor) -> bool {
    let mut map: HashMap<Pos, &Pieces> = HashMap::new();
    is_pos_potentially_checked(pos, board, side, map)
}

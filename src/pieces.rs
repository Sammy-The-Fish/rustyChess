use std::{collections::HashMap};

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
        has_just_jumped: bool,
    },
    Rook {
        side: PieceColor,
        is_first_move: bool,
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
        is_first_move: bool,
    },
}

pub enum PotentialMoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion
}



pub enum Promotion {
    Queen,
    Rook,
    Knight,
    Bishop
}


pub struct PotentialMove {
    pub destination: Pos,
    pub move_type: PotentialMoveType,
}



pub struct Move {
    pub to: Pos,
    pub from: Pos,
    pub move_type: MoveType
}

pub enum MoveType{
    Normal,
    Castle,
    EnPassant,
    Promotion { piece: Promotion }
}

impl PotentialMove {
    fn new_normal(destination: Pos) -> PotentialMove {
        PotentialMove {
            destination,
            move_type: PotentialMoveType::Normal,
        }
    }

    // pub fn to_move(&self, from: Pos) -> Move{
    //     Move {
    //         from,
    //         to: self.destination,

    //     }
    // }
}



impl Pieces {
    pub fn symbol(&self) -> char {
        match self { 
            Pieces::Empty => ' ',
            Pieces::Pawn { side: _, .. } => 'P',
            Pieces::Rook { side: _, .. } => 'R',
            Pieces::Knight { side: _ } => 'N',
            Pieces::Bishop { side: _ } => 'B',
            Pieces::Queen { side: _ } => 'Q',
            Pieces::King { side: _, .. } => 'K',
        }
    }

    pub fn color(&self) -> Option<&PieceColor> {
        match self {
            Pieces::Empty => None,
            Pieces::Pawn { side: color, .. } => Some(color),
            Pieces::Rook { side: color, .. } => Some(color),
            Pieces::Knight { side: color } => Some(color),
            Pieces::Bishop { side: color } => Some(color),
            Pieces::Queen { side: color } => Some(color),
            Pieces::King { side: color, .. } => Some(color),
        }
    }

    pub fn find_moves(&self, board: &board::Board, pos: (usize, usize)) -> Vec<PotentialMove> {
        let mut potential_result = Vec::new();
        match self {
            Pieces::Empty => (),
            Pieces::Pawn {
                side,
                is_first_move,
                ..
            } => {
                // do pawns move up or down
                let direction = {
                    if *side == PieceColor::White {
                        (1, 0)
                    } else {
                        (-1, 0)
                    }
                };
                let promotion_rank = match side {
                                PieceColor::Black => 0,
                                PieceColor::White => 7,
                            };

                // check in front
                let mut piece_in_front = true;
                if let Some(target_x) = pos.0.checked_add_signed(direction.0) {
                    if target_x < 8 {
                        let target = board.get_piece_at_pos((target_x, pos.1));
                        if target.color() == None {
                            piece_in_front = false;
                            if target_x == promotion_rank {
                                potential_result.push(PotentialMove { destination: (target_x, pos.1), move_type: PotentialMoveType::Promotion  });
                            }else {
                                potential_result.push(PotentialMove::new_normal((target_x, pos.1)));
                            }
                        }
                    }
                };

                // check 2 in front for fist move
                if *is_first_move && !piece_in_front{
                    if let Some(target_x) = pos.0.checked_add_signed(direction.0 * 2) {
                        if target_x < 8 {
                            let target = board.get_piece_at_pos((target_x, pos.1));
                            if target.color() == None {
                                potential_result.push(PotentialMove::new_normal((target_x, pos.1)));
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
                            if target_x == promotion_rank {
                                potential_result.push(PotentialMove{destination: (target_x, target_y), move_type: PotentialMoveType::Promotion });
                            }else {
                                potential_result.push(PotentialMove::new_normal((target_x, target_y)));
                            }
                        }
                    }
                }

                // check for en passant
                let enpassant_checks = [(0, -1), (0, 1)];
                for check in enpassant_checks {
                    // ensures targets cannot be < 0
                    let Some(target_pos) = board::add_positions(pos, check) else {
                        continue;
                    };

                    let target = board.get_piece_at_pos(target_pos);
                    match target {
                        Pieces::Pawn {
                            side: other_side,
                            is_first_move: _,
                            has_just_jumped: has_other_just_jumped,
                        } => {
                            if *has_other_just_jumped && *other_side != *side {
                                let move_target = board::add_positions(target_pos, direction);
                                
                                // en passant should only occur in the middle of the board, therefore unwrapping should never lead to an error
                                // checked square should also always be clear, therefore also does not need to be checked
                                potential_result.push(PotentialMove { destination: move_target.unwrap(), move_type: PotentialMoveType::EnPassant });
                            }
                        }
                        _ => (),
                    }
                }
            }
            Pieces::Rook {
                side,
                ..
            } => {
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
                                potential_result.push(PotentialMove::new_normal((target_x, target_y)));
                            }
                        } else {
                            potential_result.push(PotentialMove::new_normal((target_x, target_y)));
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
            Pieces::King {
                side,
                is_first_move,
            } => {
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
                                potential_result.push(PotentialMove::new_normal(square));
                            }
                        }
                        _ => (),
                    }
                    if let Some(color) = target.color() {
                        if color != side {
                            potential_result.push(PotentialMove::new_normal(square));
                        }
                    }
                }

                // castling
                if *is_first_move {
                    // scan in both horizontal directions to check for rooks
                    let dirs = [(0, 1), (0, -1)];

                    for dir in dirs {
                        let mut counter = 0;
                        'scan: loop {
                            counter += 1;
                            let piece_move = (dir.0 * counter, dir.1 * counter);

                            let Some(target) = board::add_positions(pos, piece_move) else {
                                break 'scan;
                            };

                            let piece = board.get_piece_at_pos(target);
                            match piece {
                                Pieces::Empty => {
                                    continue;
                                }
                                Pieces::Rook {
                                    side: piece_side,
                                    is_first_move,
                                } => {
                                    if side == piece_side && *is_first_move {
                                        potential_result.push(PotentialMove {
                                            destination: board::add_positions(
                                                pos,
                                                (dir.0 * 2, dir.1 * 2),
                                            )
                                            .unwrap(),
                                            move_type: PotentialMoveType::Castle,
                                        })
                                    }
                                }
                                _ => {
                                    break 'scan;
                                }
                            }
                        }
                    }
                }
            }
        }

        // for each possible move modify the board and check if it results in the king being in check
        let mut result = Vec::new();
        for piece_move in potential_result {
            let mut modification: HashMap<Pos, &Pieces> = HashMap::new();
            let piece = board.get_piece_at_pos(pos);
            let color = self.color().unwrap();

            // if we are moving the king, use the updated king position
            let king_pos = match piece {
                Pieces::King { .. } => piece_move.destination,
                _ => board.get_king_position(color),
            };

            // add new place to the hashmap
            modification.insert(piece_move.destination, piece);
            // replace old pos with empty
            modification.insert(pos, &Pieces::Empty);

            if !is_pos_potentially_checked(king_pos, board, color, modification) {
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
    ) -> Vec<PotentialMove> {
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
                        result.push(PotentialMove::new_normal((target_x, target_y)));
                    }
                    break 'scan;
                } else {
                    result.push(PotentialMove::new_normal((target_x, target_y)));
                }
            }
        }
        result
    }

    pub fn on_move(&mut self, from: Pos, to: Pos) {
        match self {
            Pieces::Pawn {
                side: _,
                is_first_move,
                has_just_jumped,
            } => {
                if *has_just_jumped {
                    *has_just_jumped = false;
                }

                let diff = (from.0 as isize - to.0 as isize).abs();
                if *is_first_move && diff == 2 {
                    *has_just_jumped = true;
                }

                *is_first_move = false;
                
            }
            Pieces::Rook {
                side: _,
                is_first_move,
            } => {
                *is_first_move = false;
            }
            Pieces::King {
                side: _,
                is_first_move,
            } => {
                *is_first_move = false;
            }
            _ => (),
        }
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
    // check for pawns
    let direction = { if *side == PieceColor::White { 1 } else { -1 } };

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
            match target {
                Pieces::Queen { side: piece_side } => {
                    if piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::Rook {
                    side: piece_side, ..
                } => {
                    if piece_side != side {
                        return true;
                    }
                    break;
                }
                Pieces::King {
                    side: piece_side, ..
                } => {
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
                }
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
                Pieces::King {
                    side: piece_side, ..
                } => {
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
                }
            }
        }
    }

    false
}

pub fn is_pos_checked(pos: Pos, board: &Board, side: &PieceColor) -> bool {
    let map: HashMap<Pos, &Pieces> = HashMap::new();
    is_pos_potentially_checked(pos, board, side, map)
}

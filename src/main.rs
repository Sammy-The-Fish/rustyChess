use std::io::{self, Write};

use colored::Colorize;
use pieces::{PieceColor, Pieces};

use crate::pieces::{Move, MoveType, Promotion};

mod board;
mod pieces;

const SIZE: usize = 8;
type Pos = (usize, usize);

fn main() {
    // let mut board = [[Pieces::Empty; SIZE]; SIZE];

    let mut turn = PieceColor::White;

    let mut board = board::Board::new();

    let result = loop {
        // check for check
        let mut checked = false;
        let king = board.get_king_position(&turn);
        if pieces::is_pos_checked(king, &board, &turn) {
            checked = true
        }

        let mut valid_moves = false;
        // check for legal moves
        for piece in board.get_all_pieces(&turn) {
            let piece_moves = board.get_piece_at_pos(piece).find_moves(&board, piece);

            if piece_moves.len() > 0 {
                valid_moves = true;
                break;
            }
        }

        // check win cons
        // checkmate
        if checked && !valid_moves {
            match turn {
                PieceColor::Black => break Result::WhiteWin,
                PieceColor::White => break Result::BlackWin,
            }
        }

        // stalemate
        if !valid_moves {
            break Result::Draw;
        }

        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        println!("{}", get_header(&turn, checked));
        board.print_board();

        // select piece to move
        let selected_square: (usize, usize) = loop {
            let mut input = String::new();
            print!("input piece to move >> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();

            let Some(pos) = parse_input(&input.trim()) else {
                println!("{}", "invalid input".red());
                continue;
            };

            let piece = board.get_piece_at_pos(pos);

            if let Some(color) = piece.color() {
                if *color == turn {
                    break pos;
                } else {
                    println!("{}", "must select your own piece".red());
                    continue;
                }
            }
            println!("{}", "must select a piece".red());
        };

        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // find places to move
        let moves = board
            .get_piece_at_pos(selected_square)
            .find_moves(&board, selected_square);
        if moves.len() == 0 {
            continue;
        }

        let mut destinations = Vec::new();

        for piece_move in &moves {
            destinations.push(piece_move.destination);
        }

        board.print_board_highlighted(&destinations);

        let result = 'outer: loop {
            print!("where to move  (blank to change piece)>> ",);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().len() == 0 {
                break None;
            }

            let Some(pos) = parse_input(&input.trim()) else {
                println!("{}", "invalid input".red());
                continue;
            };

            for piece_move in &moves {
                if piece_move.destination == pos {
                    break 'outer  Some(piece_move)
                }
            }

            println!("{}", "cannot move there".red());
        };

        let Some(selected_move) = result else {
            continue;
        };


        let move_type = match selected_move.move_type {
            pieces::PotentialMoveType::Normal => MoveType::Normal,
            pieces::PotentialMoveType::Castle => MoveType::Castle,
            pieces::PotentialMoveType::EnPassant => MoveType::EnPassant,
            pieces::PotentialMoveType::Promotion => {
                println!("{}", "------PROMOOTION-------".green());
                loop {
                    print!("what to promote to (R, N, B, Q)>> ");
                    io::stdout().flush().unwrap();
                                let mut input = String::new();

                    io::stdin().read_line(&mut input).unwrap();

                if input.trim().len() != 1 {
                    println!("{}", "invalid input".red());
                    continue;
                }

                match input.chars().next().unwrap() {
                    'R' => {
                        break MoveType::Promotion { piece: Promotion::Rook };
                    },
                    'N' => {
                        break MoveType::Promotion { piece: Promotion::Knight };
                    },
                    'B' => {
                        break MoveType::Promotion { piece: Promotion::Bishop };
                    },
                    'Q' => {
                        break MoveType::Promotion { piece: Promotion::Queen };
                    }
                    _ => {
                        println!("{}", "invalid piece".red());
                    }
                }

                }
            },
        };

        let current_move = Move {
            from: selected_square,
            to: selected_move.destination,
            move_type: move_type
        };

        board.move_piece(current_move);

        turn = turn.other();
    };

    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    let victory_text = match result {
        Result::BlackWin => "=======BLACK VICTORY!!!!!=======".green(),
        Result::WhiteWin => "=======WHITE VICTORY!!!!!=======".green(),
        Result::Draw => "=======STALEMATE=======".blue(),
    };

    println!("{}", victory_text);
    board.print_board();
}

fn get_header(side: &PieceColor, checked: bool) -> String {
    let side = match side {
        PieceColor::Black => "Black",
        PieceColor::White => "White",
    };

    let mut check_string = "".red();
    if checked {
        check_string = "\n=== IN CHECK ===".red()
    }

    format!("========{side} to play======={check_string}",)
}

fn parse_input(input: &str) -> Option<(usize, usize)> {
    let mut chars = input.chars();

    if input.len() != 2 {
        return None;
    }

    let column = chars.nth(0).unwrap();
    let row = chars.nth(0).unwrap();

    let digit = row.to_digit(10);
    let index2 = column.to_ascii_lowercase() as usize - 'a' as usize;

    if digit == None {
        return None;
    }
    let index1 = (digit.unwrap() - 1) as usize;

    Some((index1, index2))
}

enum Result {
    BlackWin,
    WhiteWin,
    Draw,
}



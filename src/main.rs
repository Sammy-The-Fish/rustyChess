use std::io::{self, Write};

use colored::{
    Color::{self},
    Colorize,
};
use pieces::{PieceColor, Pieces};

use crate::{
    board::Board,
    pieces::{Move, MoveType, Promotion},
};

mod board;
mod pieces;

const SIZE: usize = 8;
type Pos = (usize, usize);

enum UserInput {
    Resign,
    OfferDraw,
    Move(Pos),
}

fn main() {
    preamble();

    let mut turn = PieceColor::White;

    let mut board = board::Board::new();

    let result = loop {
        // check for check
        let checked = board.is_checked(&turn);

        // check win cons
        if let Some(result) = board.check_win_cons(&turn) {
            break result;
        }

        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        println!("{}", get_header(&turn, checked));
        print_board(&board);

        // select piece to move
        let user_input = loop {
            let mut input = String::new();
            print!("input piece to move, offer draw (D) or resign (R) >> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().len() == 1 {
                match &input.chars().next().unwrap() {
                    'D' => break UserInput::OfferDraw,
                    'R' => break UserInput::Resign,
                    _ => (),
                }
            }

            let Some(pos) = parse_input(&input.trim()) else {
                println!("{}", "invalid input".red());
                continue;
            };

            let piece = board.get_piece_at_pos(pos);

            if let Some(color) = piece.color() {
                if *color == turn {
                    break UserInput::Move(pos);
                } else {
                    println!("{}", "must select your own piece".red());
                    continue;
                }
            }

            println!("{}", "must select a piece".red());
        };

        let selected_square = match user_input {
            UserInput::Resign => match turn {
                PieceColor::Black => break GameResult::WhiteWin,
                PieceColor::White => break GameResult::BlackWin,
            },
            UserInput::OfferDraw => {
                let agreed = loop {
                    print!("DO BOTH PLAYERS AGREE TO A DRAW (Y/N) >> ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();

                    if input.trim().len() != 1 {
                        continue;
                    }

                    match input.chars().next().unwrap().to_ascii_uppercase() {
                        'Y' => {
                            break true;
                        }
                        'N' => {
                            break false;
                        }
                        _ => (),
                    }
                    println!("{}", "indalid input".red());
                };
                if agreed {
                    break GameResult::Draw;
                } else {
                    continue;
                }
            }
            UserInput::Move(pos) => pos,
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

        print_board_highlighted(&board, &destinations);

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
                    break 'outer Some(piece_move);
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
                            break MoveType::Promotion {
                                piece: Promotion::Rook,
                            };
                        }
                        'N' => {
                            break MoveType::Promotion {
                                piece: Promotion::Knight,
                            };
                        }
                        'B' => {
                            break MoveType::Promotion {
                                piece: Promotion::Bishop,
                            };
                        }
                        'Q' => {
                            break MoveType::Promotion {
                                piece: Promotion::Queen,
                            };
                        }
                        _ => {
                            println!("{}", "invalid piece".red());
                        }
                    }
                }
            }
        };

        let current_move = Move {
            from: selected_square,
            to: selected_move.destination,
            move_type: move_type,
        };

        board.move_piece(current_move);

        turn = turn.other();
    };

    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    let victory_text = match result {
        GameResult::BlackWin => "=======BLACK VICTORY!!!!!=======".green(),
        GameResult::WhiteWin => "=======WHITE VICTORY!!!!!=======".green(),
        GameResult::Draw => "=======STALEMATE=======".blue(),
    };

    println!("{}", victory_text);
    print_board(&board);
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

fn parse_input(input: &str) -> Option<Pos> {
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

enum GameResult {
    BlackWin,
    WhiteWin,
    Draw,
}

fn preamble() {
    print!("\x1B[2J\x1B[1;1H");

    println!(
        "{}",
        r"
 $$$$$$\  $$\   $$\ $$$$$$$$\  $$$$$$\   $$$$$$\  
$$  __$$\ $$ |  $$ |$$  _____|$$  __$$\ $$  __$$\ 
$$ /  \__|$$ |  $$ |$$ |      $$ /  \__|$$ /  \__|
$$ |      $$$$$$$$ |$$$$$\    \$$$$$$\  \$$$$$$\  
$$ |      $$  __$$ |$$  __|    \____$$\  \____$$\ 
$$ |  $$\ $$ |  $$ |$$ |      $$\   $$ |$$\   $$ |
\$$$$$$  |$$ |  $$ |$$$$$$$$\ \$$$$$$  |\$$$$$$  |
 \______/ \__|  \__|\________| \______/  \______/ 
                                      
    "
        .blue()
    );

    print!("press ENTER to start!");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn print_board_highlighted(board: &Board, highlight: &[(usize, usize)]) {
    let mut square: u32 = 0;

    println!("   A  B  C  D  E  F  G  H");

    // prints from bottom to top
    for i in (0..SIZE).rev() {
        // offset grid by 1 each row to get proper checkerboard
        square += 1;
        print!("{} ", i + 1);

        for j in 0..SIZE {
            let piece = board.get_piece_at_pos((i, j));

            // get colour
            let text_color = match piece.color() {
                Some(color) => match color {
                    PieceColor::Black => Color::Black,
                    PieceColor::White => Color::White,
                },
                None => Color::White,
            };
            let text = format!(" {} ", piece.symbol());

            // select background color
            let mut background = Color::Cyan;
            if (square % 2) == 0 {
                background = Color::Blue;
            }

            for pos in highlight {
                // adjust position due to printing from bottom to top
                if (i, j) == *pos {
                    background = Color::Red
                }
            }

            print!("{}", text.color(text_color).on_color(background));

            square += 1;
        }
        print!("\n");
    }
}

fn print_board(board: &Board) {
    print_board_highlighted(board, &[]);
}

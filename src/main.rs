use std::io::{self, Write};

use crate::{
    board::{Board, DrawReason, GameResult},
    pieces::{Move, MoveType, Promotion},
};
use colored::{
    Color::{self},
    Colorize,
};
use pieces::{PieceColor, Pieces};

mod board;
mod pieces;

const SIZE: usize = 8;
type Pos = (usize, usize);

enum UserInput {
    Resign,
    OfferDraw,
    Move(Pos),
}

impl Pieces {
    pub fn display(&self) -> char {
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
}

fn main() {
    preamble();

    let mut turn = PieceColor::White;

    let mut board = board::Board::new();

    let result = loop {
        // check for check
        let checked = board.is_checked(&turn);

        let border_color = if checked { Color::Red } else { Color::White };

        let checked_message = if checked { " - in check" } else { "" };

        // check win cons
        if let Some(result) = board.check_win_cons(&turn) {
            break result;
        }

        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        let title = match turn {
            PieceColor::White => "white to play",
            PieceColor::Black => "black to play",
        };
        print_board(
            &board,
            &format!("{}{}", &title, &checked_message),
            border_color,
        );

        // select piece to move
        let user_input = loop {
            let mut input = String::new();
            print!(
                "input piece to move e.g. a1, claim or offer a draw (D), resign (R) or help (H) >> "
            );
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().len() == 1 {
                match &input.chars().next().unwrap().to_ascii_uppercase() {
                    'D' => break UserInput::OfferDraw,
                    'R' => break UserInput::Resign,
                    'H' => {
                        println!(
                            "HELP:
To Move:
 - first select a piece to move e.g a2
 - select a piece to move to, all possile squares to move are highlighted in red
In addition to moving you can resign, this will make your opponent win.
Or you can claim / offer a draw, this will draw the game if the followinf conditions are met:
 - the position currently in has been repeated 3 times in the game so far
 - it has been 50 moves since the last piece was captured
if these conditions are not met, the opponent will be asked if they agree to a draw.
Finally a draw will be mandated if:
 - the position currently in has been repeated 5 times in the game so far
 - it has been 75 moves since the last piece was captured"
                        );
                        continue;
                    }
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
                if let Some(result) = board.check_claimed_draw_cons() {
                    break result;
                }

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
                    break GameResult::Draw {
                        reason: DrawReason::Agreed,
                    };
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

        print_board_highlighted(&board, &destinations, "select destination", border_color);

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

    let victory_color = match result {
        GameResult::Draw { .. } => Color::Blue,
        _ => Color::Green,
    };

    let victory_text = match result {
        GameResult::BlackWin => "BLACK VICTORY!!!!!",
        GameResult::WhiteWin => "WHITE VICTORY!!!!!",
        GameResult::Draw{reason} => {
            let draw_text = "DRAW - ";

            let reason_text = match reason {
                DrawReason::Stalemate => "Stalemate",
                DrawReason::Agreed => "Agreed",
                DrawReason::ThreefoldRepeat => "Threefold Repeat",
                DrawReason::FivefoldRepeat => "Fivefold Repeat",
                DrawReason::InsufficientMaterial => "Insufficient Material",
                DrawReason::FiftyMoves => "50 Moves",
                DrawReason::SeventyFiveMoves => "75 Moves",
            };


            &format!("{}{}", draw_text, reason_text)
        }
    };

    // println!("{}", victory_text);
    print_board(&board, &victory_text.color(victory_color), victory_color);
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

fn print_board_highlighted(
    board: &Board,
    highlight: &[(usize, usize)],
    title: &str,
    border_color: Color,
) {
    let mut square: u32 = 0;
    const BORDER_LENGTH: i32 = 30;
    let wings = (BORDER_LENGTH - title.len() as i32) / 2;

    print!("{}", "┏".color(border_color));

    for _ in 1..wings {
        print!("{}", "━".color(border_color));
    }
    print!("{}", title.color(border_color));

    for _ in 1..wings {
        print!("{}", "━".color(border_color));
    }
    if title.len() % 2 == 1 {
        print!("{}", "━".color(border_color));
    }

    print!("{}", "┓\n".color(border_color));

    println!(
        "{}    A  B  C  D  E  F  G  H  {}",
        "┃".color(border_color),
        "┃".color(border_color)
    );

    // prints from bottom to top
    for i in (0..SIZE).rev() {
        // offset grid by 1 each row to get proper checkerboard
        square += 1;
        print!("{} {} ", "┃".color(border_color), i + 1);
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
            let text = format!(" {} ", piece.display());

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
        print!("{}", " ┃\n".color(border_color));
    }
    print!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\n".color(border_color));
}

fn print_board(board: &Board, title: &str, border_color: Color) {
    print_board_highlighted(board, &[], title, border_color);
}

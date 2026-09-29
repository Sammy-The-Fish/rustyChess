use crate::{PieceColor, Pieces, Pos, SIZE, pieces};
use colored::{Color, Colorize};

pub struct Board {
    board: [[Pieces; SIZE]; SIZE],
}


impl Board {
    pub fn new() -> Board {
        let mut board: [[Pieces; SIZE]; SIZE] =
            std::array::from_fn(|_| std::array::from_fn(|_| Pieces::Empty));

        // set up black back rank
        board[0][0] = Pieces::Rook {
            side: (PieceColor::White),
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
        };
        board[0][5] = Pieces::Bishop {
            side: (PieceColor::White),
        };
        board[0][6] = Pieces::Knight {
            side: (PieceColor::White),
        };
        board[0][7] = Pieces::Rook {
            side: (PieceColor::White),
        };

        // set up white pawns
        for i in 0..SIZE {
            board[1][i] = Pieces::Pawn {
                side: (PieceColor::White),
                is_first_move: true,
            };
        }

        // set up white back rank
        board[SIZE - 1][0] = Pieces::Rook {
            side: (PieceColor::Black),
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
        };
        board[SIZE - 1][5] = Pieces::Bishop {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][6] = Pieces::Knight {
            side: (PieceColor::Black),
        };
        board[SIZE - 1][7] = Pieces::Rook {
            side: (PieceColor::Black),
        };

        // set up black pawns
        for i in 0..SIZE {
            board[SIZE - 2][i] = Pieces::Pawn {
                side: (PieceColor::Black),
                is_first_move: true,
            };
        }

        // board modifications to make testing easier
        board[6][4] = Pieces::Queen { side: PieceColor::Black };
        board[0][3] = Pieces::Empty;
        board[0][5] = Pieces::Empty;


        Board { board }

        
    }
    // might implement get_king better later
    pub fn get_king_position(&self, side: &PieceColor) -> Pos {
        for i in 0..SIZE {
            for j in 0..SIZE {
                let piece = self.get_piece_at_pos((i,j));
                match piece {
                    Pieces::King { side: piece_side} => {
                        if piece_side == side {
                            return (i, j)
                        }
                    },
                    _ => ()
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
                let piece = self.get_piece_at_pos((i,j));
                if let Some(color) = piece.color() {
                    if color == side {
                        result.push((i, j));
                    }
                }
            }
        }
        result
    }

    pub fn print_board_highlighted(&self, highlight: &[(usize, usize)]) {
        let mut square: u32 = 0;

        println!("   A  B  C  D  E  F  G  H");

        // prints from bottom to top
        for i in (0..SIZE).rev() {
            // offset grid by 1 each row to get proper checkerboard
            square += 1;
            print!("{} ", i + 1);

            for j in (0..SIZE) {
                let piece = self.get_piece_at_pos((i, j));

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

    pub fn print_board(&self) {
        self.print_board_highlighted(&[]);
    }

    pub fn get_piece_at_pos(&self, pos: (usize, usize)) -> &Pieces {
        &self.board[pos.0][pos.1]
    }

    pub fn get_mut_piece_at_pos(&mut self, pos: (usize, usize)) -> &mut Pieces {
        &mut self.board[pos.0][pos.1]
    }

    

    pub fn move_piece(&mut self, from: Pos, to: Pos) {
        let from_piece = self.get_mut_piece_at_pos(from);
        from_piece.on_move();

        let from_piece = self.get_piece_at_pos(from);
        let to_piece  = self.get_piece_at_pos(to);
        let from_colour = from_piece.color().unwrap();

        if let Some(to_color) = to_piece.color() {
            if from_colour == to_color {
                panic!()
            }
        }


        let from_piece = std::mem::replace(&mut self.board[from.0][from.1], Pieces::Empty);

        self.board[to.0][to.1] = from_piece;
    }
}


pub fn add_positions(pos1: Pos, pos2: (isize, isize)) -> Option<Pos> {
    let result = ((pos1.0 as isize + pos2.0), (pos1.1 as isize + pos2.1));

    if result.0 < 0 || result.0 >= SIZE as isize{
        return None
    }
    if result.1 < 0 || result.1 >= SIZE as isize {
        return None
    }

    Some((result.0 as usize, result.1 as usize))

}
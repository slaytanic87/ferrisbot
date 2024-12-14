use core::fmt;

const DIAGONAL_ONLY_CONSTRAINT: [i8; 14] = [7, 9, 14, 18, 21, 27, 28, 35, 36, 42, 45, 49, 54, 63];
const ALL_DIRECTION_ONE_STEP_CONTRAINT: [i8; 4] = [1, 7, 8, 9];
const JUMP_CONTRAINT: [i8; 3] = [10, 15, 17];
const PAWN_WHITE_CONSTRAINT: [i8; 3] = [-7, -8, -9];
const PAWN_BLACK_CONSTRAINT: [i8; 3] = [7, 8, 9];

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum FigureType {
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Pawn(Color),
    Queen(Color),
    King(Color),
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy)]
#[repr(i8)]
pub enum Coord {
    A(i8),
    B(i8),
    C(i8),
    D(i8),
    E(i8),
    F(i8),
    G(i8),
    H(i8),
}

#[derive(Clone)]
pub struct Step {
    pub start: Coord,
    pub target: Coord,
}

#[derive(Clone, Copy)]
pub struct ChessMan {
    pub identity: FigureType,
    pub is_first_turn: bool,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub figure: Option<ChessMan>,
    pub number: i8,
}

impl Coord {
    /**
     * return (row, col)
     */
    pub fn extract_2d_coordinate(self) -> (usize, usize) {
        match self {
            Coord::A(col) => (0, col as usize),
            Coord::B(col) => (1, col as usize),
            Coord::C(col) => (2, col as usize),
            Coord::D(col) => (3, col as usize),
            Coord::E(col) => (4, col as usize),
            Coord::F(col) => (5, col as usize),
            Coord::G(col) => (6, col as usize),
            Coord::H(col) => (7, col as usize),
        }
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let enum_value = match *self {
            Coord::A(_) => 'A',
            Coord::B(_) => 'B',
            Coord::C(_) => 'C',
            Coord::D(_) => 'D',
            Coord::E(_) => 'E',
            Coord::F(_) => 'F',
            Coord::G(_) => 'G',
            Coord::H(_) => 'H',
        };
        write!(formatter, "{}", enum_value)
    }
}

impl FigureType {
    pub fn unwrap_color(self) -> Color {
        match self {
            FigureType::Knight(color_value) => color_value,
            FigureType::Bishop(color_value) => color_value,
            FigureType::Rook(color_value) => color_value,
            FigureType::Pawn(color_value) => color_value,
            FigureType::Queen(color_value) => color_value,
            FigureType::King(color_value) => color_value,
        }
    }
}

impl ChessMan {
    pub fn is_step_allowed(&self, start_field_num: i8, target_field_num: i8) -> bool {
        let difference_abs: i8 = i8::abs(start_field_num - target_field_num);
        match self.identity {
            FigureType::Knight(_) => JUMP_CONTRAINT.contains(&difference_abs),
            FigureType::Bishop(_) => DIAGONAL_ONLY_CONSTRAINT.contains(&difference_abs),
            FigureType::King(_) => ALL_DIRECTION_ONE_STEP_CONTRAINT.contains(&difference_abs),
            FigureType::Queen(_) => true,
            FigureType::Pawn(_) => {
                let difference: i8 = start_field_num - target_field_num;
                match self.identity.unwrap_color() {
                    Color::White => {
                        if self.is_first_turn {
                            // 16 only allowed on first turn
                            return PAWN_WHITE_CONSTRAINT.contains(&difference)
                                && difference == -16;
                        }
                        PAWN_WHITE_CONSTRAINT.contains(&difference)
                    }
                    Color::Black => {
                        if self.is_first_turn {
                            // 16 only allowed on first turn
                            return PAWN_BLACK_CONSTRAINT.contains(&difference) && difference == 16;
                        }
                        PAWN_BLACK_CONSTRAINT.contains(&difference)
                    }
                }
            }
            FigureType::Rook(_) => !DIAGONAL_ONLY_CONSTRAINT.contains(&difference_abs),
        }
    }
}

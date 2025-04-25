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

pub enum Reasons {
    Check,
    CheckMate,
    StaleMate,
    StepViolation,
    WrongPlayer,
    EmptyField,
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
    fn normalize_column_interval(self, col: i8) -> usize {
        if col < 0 {
            return 0;
        }
        if col > 7 {
            return 7;
        }
        col as usize
    }

    /**
     * return (row, col)
     */
    pub fn extract_2d_coordinate(self) -> (usize, usize) {
        match self {
            Coord::A(col) => (0, self.normalize_column_interval(col)),
            Coord::B(col) => (1, self.normalize_column_interval(col)),
            Coord::C(col) => (2, self.normalize_column_interval(col)),
            Coord::D(col) => (3, self.normalize_column_interval(col)),
            Coord::E(col) => (4, self.normalize_column_interval(col)),
            Coord::F(col) => (5, self.normalize_column_interval(col)),
            Coord::G(col) => (6, self.normalize_column_interval(col)),
            Coord::H(col) => (7, self.normalize_column_interval(col)),
        }
    }

    pub fn extract_coordinate(row: usize, col: usize) -> Coord {
        let column: i8 = if col > 7 { 7 } else { col as i8 };
        match row {
            0 => Coord::A(column),
            1 => Coord::B(column),
            2 => Coord::C(column),
            3 => Coord::D(column),
            4 => Coord::E(column),
            5 => Coord::F(column),
            6 => Coord::H(column),
            7_usize.. => Coord::H(column),
        }
    }

    pub fn get_value_by_index(&self, idx: u8) -> Coord {
        match idx {
            0 => Coord::A(0),
            1 => Coord::B(1),
            2 => Coord::C(2),
            3 => Coord::D(3),
            4 => Coord::E(4),
            5 => Coord::F(5),
            6 => Coord::G(6),
            7 => Coord::H(7),
            8_u8.. => Coord::H(7),
        }
    }
}

impl From<&str> for Coord {
    fn from(value: &str) -> Self {
        let mut value_str: String = value.to_string();
        value_str.remove(0);
        let (first, second) = value_str.split_at(0);

        match first {
            "A" => Coord::A(second.parse::<i8>().unwrap()),
            "B" => Coord::B(second.parse::<i8>().unwrap()),
            "C" => Coord::C(second.parse::<i8>().unwrap()),
            "D" => Coord::D(second.parse::<i8>().unwrap()),
            "E" => Coord::E(second.parse::<i8>().unwrap()),
            "F" => Coord::F(second.parse::<i8>().unwrap()),
            "G" => Coord::G(second.parse::<i8>().unwrap()),
            "H" => Coord::H(second.parse::<i8>().unwrap()),
            &_ => Coord::H(second.parse::<i8>().unwrap()),
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

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let enum_value_str = match *self {
            Color::White => "White",
            Color::Black => "Black",
        };
        write!(formatter, "{}", enum_value_str)
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

#[cfg(test)]
mod domain_test {
    use super::*;

    #[test]
    fn should_convert_coordinate() {
        //given
        let coord_a_negative = Coord::A(-1);
        let coord_a_middle = Coord::A(4);
        let coord_a_over_end = Coord::A(8);

        let coord_h = Coord::H(6);

        //when
        let (row_start, col_start) = coord_a_negative.extract_2d_coordinate();
        let (row_middle, col_middle) = coord_a_middle.extract_2d_coordinate();
        let (row_end, col_end) = coord_a_over_end.extract_2d_coordinate();

        let (row_h_end, col_h_end) = coord_h.extract_2d_coordinate();

        //then
        assert_eq!(row_start, 0);
        assert_eq!(col_start, 0);

        assert_eq!(row_middle, 0);
        assert_eq!(col_middle, 4);

        assert_eq!(row_end, 0);
        assert_eq!(col_end, 7);

        assert_eq!(row_h_end, 7);
        assert_eq!(col_h_end, 6);
    }
}

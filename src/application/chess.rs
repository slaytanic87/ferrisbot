use std::cmp::Ordering;
use std::collections::HashMap;

use super::{chess_domain_data::Color, Cell, ChessMan, Coord, FigureType, Step};

const FIELD: [[Option<FigureType>; 8]; 8] = [
    [
        Some(FigureType::Rook(Color::Black)),
        Some(FigureType::Knight(Color::Black)),
        Some(FigureType::Bishop(Color::Black)),
        Some(FigureType::Queen(Color::Black)),
        Some(FigureType::King(Color::Black)),
        Some(FigureType::Bishop(Color::Black)),
        Some(FigureType::Knight(Color::Black)),
        Some(FigureType::Rook(Color::Black)),
    ],
    [
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
        Some(FigureType::Pawn(Color::Black)),
    ],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [None, None, None, None, None, None, None, None],
    [
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
        Some(FigureType::Pawn(Color::White)),
    ],
    [
        Some(FigureType::Rook(Color::White)),
        Some(FigureType::Knight(Color::White)),
        Some(FigureType::Bishop(Color::White)),
        Some(FigureType::Queen(Color::White)),
        Some(FigureType::King(Color::White)),
        Some(FigureType::Bishop(Color::White)),
        Some(FigureType::Knight(Color::White)),
        Some(FigureType::Rook(Color::White)),
    ],
];

/**
* 1  2  3  4   5  6  7  8
* 9  10 11 12 13 14 15 16
* 17 18 19 20 21 22 23 24
* 25 26 27 28 29 30 31 32
* 33 34 35 36 37 38 39 40
* 41 42 43 44 45 46 47 48
* 49 50 51 52 53 54 55 56
* 57 58 59 60 61 62 63 64
*
* Constraint = start field number - target field number
*
* Rook   invert(Bishop)
* Knight 10, 15, 17
* Bishop 7, 9, 14, 18, 21, 27, 28, 35, 36, 42, 45, 49, 54, 63
* Queen  no constraint is needed
* King   1, 7, 8, 9
* Pawn white -7, -8, -9, (-16)'1
* Pawn black  7,  8,  9, (16)'1
* '1 = only allowed on first move
*
**/
#[derive(Clone)]
pub struct ChessGame {
    pub fields: Vec<Vec<Cell>>,
    pub current_player: Color,
    pub player_in_check: Option<Color>,
    pub figure_map: HashMap<FigureType, String>,
}

impl ChessGame {
    pub fn new_game(&mut self) {
        let mut game_field: Vec<Vec<Cell>> = Vec::new();

        for (row_index, row_item) in FIELD.iter().enumerate() {
            let mut col: Vec<Cell> = Vec::new();
            for (col_index, col_item) in row_item.iter().enumerate() {
                let field_number: i8 = (row_index as i8) * (col_index as i8) - 1;
                let cell: Cell = match col_item {
                    Some(figure) => Cell {
                        figure: Some(ChessMan {
                            identity: (*figure),
                            is_first_turn: true,
                        }),
                        number: field_number,
                    },
                    None => Cell {
                        figure: None,
                        number: field_number,
                    },
                };
                col.push(cell);
            }
            game_field.push(col);
        }
        self.fields = game_field;
    }

    pub fn new() -> Self {
        let mut figure_map: HashMap<FigureType, String> = HashMap::new();
        figure_map.insert(FigureType::King(Color::Black), "♚".to_string());
        figure_map.insert(FigureType::Queen(Color::Black), "♛".to_string());
        figure_map.insert(FigureType::Knight(Color::Black), "♞".to_string());
        figure_map.insert(FigureType::Bishop(Color::Black), "♝".to_string());
        figure_map.insert(FigureType::Rook(Color::Black), "♜".to_string());
        figure_map.insert(FigureType::Pawn(Color::Black), "♟".to_string());

        figure_map.insert(FigureType::King(Color::White), "♔".to_string());
        figure_map.insert(FigureType::Queen(Color::White), "♕".to_string());
        figure_map.insert(FigureType::Knight(Color::White), "♘".to_string());
        figure_map.insert(FigureType::Bishop(Color::White), "♗".to_string());
        figure_map.insert(FigureType::Rook(Color::White), "♖".to_string());
        figure_map.insert(FigureType::Pawn(Color::White), "♙".to_string());
        Self {
            fields: Vec::new(),
            current_player: Color::White,
            player_in_check: None,
            figure_map,
        }
    }

    fn is_way_blocked(&self, step: &Step) -> bool {
        let (row_start, col_start) = step.start.extract_2d_coordinate();
        let (row_target, col_target) = step.target.extract_2d_coordinate();
        let col_diff: i8 = col_target as i8 - col_start as i8;
        let row_diff: i8 = row_target as i8 - row_start as i8;
        let col_factor: i8 = match col_diff.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        let row_factor: i8 = match row_diff.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        let mut col_index = col_start as i8;
        let mut row_index = row_start as i8;
        while col_index != (col_target as i8 - col_factor)
            && row_index != (row_target as i8 - row_factor)
        {
            col_index += col_factor;
            row_index += row_factor;
            let current_cell: Cell = self.fields[row_index as usize][col_index as usize];
            if current_cell.figure.is_some() {
                return true;
            }
        }
        false
    }

    fn get_king_data(&self, king_color: Color) -> (Coord, Color) {
        for (row_index, row) in self.fields.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                if cell.figure.is_none() {
                    continue;
                }
                let figure: ChessMan = cell.figure.unwrap();
                match figure.identity {
                    FigureType::King(color) => {
                        if color == king_color {
                            return (Coord::extract_coordinate(row_index, col_index), king_color);
                        }
                    }
                    _ => continue,
                }
            }
        }
        panic!("King figure could not be found, this case should not be occurred!")
    }

    fn is_step_allowed(&self, step: &Step, player: &Color) -> bool {
        if *player != self.current_player {
            return false;
        }

        let (row_start, col_start) = step.start.extract_2d_coordinate();
        let (row_target, col_target) = step.target.extract_2d_coordinate();
        let start_cell: Cell = self.fields[row_start][col_start];
        let target_cell: Cell = self.fields[row_target][col_target];

        if let Some(target_figure) = target_cell.figure {
            if target_figure.identity.unwrap_color() == *player {
                return false;
            }
        }

        let player_self_in_check = match self.player_in_check {
            Some(color) => color == *player,
            None => true,
        };

        match start_cell.figure {
            Some(figure) => {
                if player_self_in_check && figure.identity != FigureType::King(*player) {
                    return false;
                }
                figure.is_step_allowed(start_cell.number, target_cell.number)
                    && self.is_way_blocked(step)
            }
            None => false,
        }
    }

    fn move_figure(&mut self, step: &Step) {
        self.current_player = match self.current_player {
            Color::Black => Color::White,
            Color::White => Color::Black,
        };

        let (row_start, col_start) = step.start.extract_2d_coordinate();
        let start_cell: Cell = self.fields[row_start][col_start];

        if start_cell.figure.is_none() {
            return;
        }

        start_cell.figure.unwrap().is_first_turn = false;
        self.fields[row_start][col_start] = Cell {
            figure: None,
            number: start_cell.number,
        };
        let (row_target, col_target) = step.target.extract_2d_coordinate();
        self.fields[row_target][col_target] = start_cell;
    }

    pub fn player_in_check(&self, me_self: bool) -> Option<Color> {
        let king_color: Color = if !me_self {
            match self.current_player {
                Color::White => Color::Black,
                Color::Black => Color::White,
            }
        } else {
            self.current_player
        };
        let (coord_king, _) = self.get_king_data(king_color);
        for (row_index, row) in self.fields.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                if cell.figure.is_none() {
                    continue;
                }
                let pointed_player: ChessMan = cell.figure.unwrap();
                if pointed_player.identity.unwrap_color() != self.current_player {
                    continue;
                }
                let from: Coord = Coord::extract_coordinate(row_index, col_index);
                let step = Step {
                    start: from,
                    target: coord_king,
                };
                if self.is_step_allowed(&step, &pointed_player.identity.unwrap_color()) {
                    return Some(king_color);
                }
            }
        }
        None
    }

    pub fn render_current_board(&self) -> String {
        let mut rendered_field = String::from(" |A|B|C|D|E|F|G|H| \n");
        let mut field_type_black: bool = true;
        for (row_index, row_item) in self.fields.iter().enumerate() {
            rendered_field.push_str(format!("{}|", row_index).as_ref());
            for col_item in row_item.iter() {
                match col_item.figure {
                    Some(figure) => {
                        let figure: Option<&String> = self.figure_map.get(&figure.identity);
                        if let Some(figure_str) = figure {
                            rendered_field.push_str(format!("{}|", figure_str).as_ref());
                        } else {
                            rendered_field.push_str("E|");
                        }
                    }
                    None => {
                        if field_type_black {
                            rendered_field.push_str("#|");
                        } else {
                            rendered_field.push_str(" |");
                        }
                    }
                }
                field_type_black = !field_type_black;
            }
            field_type_black = !field_type_black;
            rendered_field.push_str(" \n");
        }
        rendered_field
    }

    pub fn make_move(&mut self, color: &Color, from: &str, to: &str) -> String {
        let start: Coord = from.into();
        let target: Coord = to.into();
        let step = Step { start, target };
        let is_allowed: bool = self.is_step_allowed(&step, color);
        if !is_allowed {
            return "Step is not allowed!".to_string();
        }
        self.move_figure(&step);
        let opponent_player_in_check: Option<Color> = self.player_in_check(false);
        if opponent_player_in_check.is_some() {
            self.player_in_check = opponent_player_in_check;
            return format!("{} is in check!", opponent_player_in_check.unwrap());
        }
        let player_in_check: Option<Color> = self.player_in_check(true);
        if player_in_check.is_some() {
            let rollback_step = Step {
                start: target,
                target: start,
            };
            self.move_figure(&rollback_step);
            return format!(
                "step not possible {} is in check!",
                player_in_check.unwrap()
            );
        }
        "Step done!".to_string()
    }

    #[warn(dead_code)]
    pub fn is_check_mate(&self) -> bool {
        //TODO impl.
        false
    }
}

#[cfg(test)]
mod chess_game_test {

    use super::*;

    #[tokio::test]
    async fn should_render_game_field_initial_correctly() {
        //given
        let mut chess_game = ChessGame::new();
        chess_game.new_game();
        let mut result: String = " |A|B|C|D|E|F|G|H| \n".to_owned();
        result.push_str("0|♜|♞|♝|♛|♚|♝|♞|♜| \n");
        result.push_str("1|♟|♟|♟|♟|♟|♟|♟|♟| \n");
        result.push_str("2|#| |#| |#| |#| | \n");
        result.push_str("3| |#| |#| |#| |#| \n");
        result.push_str("4|#| |#| |#| |#| | \n");
        result.push_str("5| |#| |#| |#| |#| \n");
        result.push_str("6|♙|♙|♙|♙|♙|♙|♙|♙| \n");
        result.push_str("7|♖|♘|♗|♕|♔|♗|♘|♖| \n");

        //when
        let rendered: String = chess_game.render_current_board();

        //then
        assert!(!rendered.is_empty());
        assert_eq!(rendered, result);
    }
}

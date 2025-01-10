use crate::application::{ChessGame, Color, Coord, FigureType, Step};
use anyhow::anyhow;
use mobot::{Action, BotState, Event, State};
use regex::{Regex, RegexSet};
use std::{collections::HashMap, usize};

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

fn extract_cmds(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"#[a-z]+");
    match separator {
        Ok(sep) => sep.split(value).collect(),
        Err(_) => Vec::new(),
    }
}

fn extract_coordinates(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"#[A-H]{1}[0-7]{1}");
    match separator {
        Ok(sep) => sep.find_iter(value).map(|value| value.as_str()).collect(),
        Err(_) => Vec::new(),
    }
}

fn extract_color(value: &str) -> Option<Color> {
    let matcher_set = RegexSet::new([r"#white", r"#black"]);
    let matches: Vec<usize> = match matcher_set {
        Ok(set) => set
            .matches(value.to_lowercase().as_str())
            .into_iter()
            .collect(),
        Err(_) => panic!("Regex expression for extracting the color is invalid!"),
    };
    if matches.is_empty() {
        return None;
    }

    match matches[0] {
        0 => Some(Color::White),
        1 => Some(Color::Black),
        2_usize.. => None,
    }
}

#[derive(Clone, BotState)]
pub struct ChessController {
    game: ChessGame,
    figure_map: HashMap<FigureType, String>,
}

impl Default for ChessController {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessController {
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
        let mut chess_game = ChessGame::new();
        chess_game.new_game();
        Self {
            game: chess_game,
            figure_map,
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

    pub fn render_current_board(&self) -> String {
        let mut rendered_field = String::from(" |A|B|C|D|E|F|G|H| \n");
        let mut field_type_black: bool = true;
        for (row_index, row_item) in self.game.fields.iter().enumerate() {
            rendered_field.push_str(format!("{}|", row_index).as_ref());
            for (_, col_item) in row_item.iter().enumerate() {
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
        let is_allowed = self.game.is_step_allowed(&step, color);
        if !is_allowed {
            return "Step is not allowed!".to_string();
        }
        self.game.make_step(&step);
        let player_in_check: Option<Color> = self.game.is_in_check();
        let message = match player_in_check {
            Some(color) => {
                self.game.player_in_check = Some(color);
                format!("Player {} is in check!", color)
            }
            None => "Step done".to_string(),
        };
        message
    }

    pub fn new_game(&mut self) {
        self.game.new_game()
    }
}

pub async fn chess_command_handler(
    event: Event,
    state: State<ChessController>,
) -> Result<Action, anyhow::Error> {
    let command: &String = event
        .update
        .get_message()?
        .text
        .as_ref()
        .ok_or(anyhow!("Not a command"))?;
    let mut chess_controller = state.get().write().await;

    let response: String = match command.as_str() {
        "/start" | "/newgame" => {
            chess_controller.new_game();
            let board: String = chess_controller.render_current_board();
            format!("{} \n {}", board, "New game!".to_owned())
        }
        "/help" => {
            let introduction =
                "This is an asyncron chat based chess game. Type /newgame to restart the game"
                    .to_owned();
            let instruction =
                "To make a move: #move #(white or black) #(start coordinate) #(end coordinate) \n
                          e.g #move #white #A0 #A1";
            format!("{} \n {}", introduction, instruction)
        }
        _ => "Unknown command".to_owned(),
    };

    Ok(Action::ReplyText(response))
}

pub async fn chess_chat_actions(
    event: Event,
    state: State<ChessController>,
) -> Result<Action, anyhow::Error> {
    let message: String = event.update.get_message()?.clone().text.unwrap().clone();

    let cmds: Vec<&str> = extract_cmds(message.as_str());
    if cmds.is_empty() {
        return Ok(Action::Done);
    }
    let command: &&str = cmds.first().unwrap();
    match *command {
        "#move" => {
            let coordinates: Vec<&str> = extract_coordinates(message.as_str());
            if coordinates.len() <= 1 {
                return Ok(Action::ReplyText(
                    "missing start and target coordinates for a step".into(),
                ));
            }
            let mut chess_controller = state.get().write().await;
            let player_color: Option<Color> = extract_color(&message);
            if player_color.is_none() {
                return Ok(Action::ReplyMarkdown(
                    "Unknown or missing player only #white or #black is allowed".to_owned(),
                ));
            }
            let move_msg: &str =
                &chess_controller.make_move(&player_color.unwrap(), coordinates[0], coordinates[1]);
            let board: String = chess_controller.render_current_board();
            Ok(Action::ReplyMarkdown(format!("{} \n {}", board, move_msg)))
        }
        _ => Ok(Action::Done),
    }
}

#[cfg(test)]
mod controller_test {

    use super::*;

    #[tokio::test]
    async fn should_render_game_field_initial_correctly() {
        //given
        let controller = ChessController::new();
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
        let rendered: String = controller.render_current_board();

        //then
        assert!(!rendered.is_empty());
        assert_eq!(rendered, result);
    }
}

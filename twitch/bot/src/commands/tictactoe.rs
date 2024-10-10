use std::collections::HashMap;

use super::ChatCommand;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mark {
    X,
    O,
}

impl Mark {
    fn other(&self) -> Mark {
        match self {
            Mark::O => Mark::X,
            Mark::X => Mark::O,
        }
    }

    fn to_char(self) -> char {
        match self {
            Self::O => 'O',
            Self::X => 'X',
        }
    }

    fn to_value(self) -> i8 {
        match self {
            Self::O => 1,
            Self::X => -1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Tie,
    Turn(Mark),
    Winner(Mark),
}

impl State {
    fn to_mark(&self) -> Mark {
        match self {
            Self::Tie => Mark::X,
            Self::Winner(m) => *m,
            Self::Turn(m) => *m,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Board {
    marks: [Option<Mark>; 9],
}

const NO_MARK: Option<Mark> = None;
impl Board {
    fn new() -> Self {
        Board {
            marks: [NO_MARK; 9],
        }
    }

    fn print(&self) {
        for i in 0..9 {
            print!(
                "{}",
                match &self.marks[i as usize] {
                    None => ' ', //(b'0' + i) as char,
                    Some(m) => m.to_char(),
                }
            );
            if i % 3 == 2 {
                println!();
            } else {
                print!("|");
            }
        }
    }

    fn get_state(&self) -> State {
        for row in 0..3 {
            let mark = self.marks[row * 3];
            if mark.is_none() {
                continue;
            }
            if self.marks.iter().skip(row * 3).take(3).all(|&m| m == mark) {
                return State::Winner(mark.unwrap());
            }
        }
        for col in 0..3 {
            let mark = self.marks[col];
            if mark.is_none() {
                continue;
            }
            if self
                .marks
                .iter()
                .skip(col)
                .step_by(3)
                .take(3)
                .all(|&m| m == mark)
            {
                return State::Winner(mark.unwrap());
            }
        }
        for diag in 0..2 {
            let mark = &self.marks[diag * 2];
            if mark.is_none() {
                continue;
            }
            if self
                .marks
                .iter()
                .skip(diag * 2)
                .step_by(4 - diag * 2)
                .take(3)
                .all(|m| m == mark)
            {
                return State::Winner(mark.unwrap());
            }
        }

        if self.marks.iter().all(Option::is_some) {
            return State::Tie;
        }

        let r = self.marks.iter().filter(|m| m.is_some()).count();
        if r % 2 == 0 {
            State::Turn(Mark::X)
        } else {
            State::Turn(Mark::O)
        }
    }
    fn place(&mut self, i: usize) -> Option<()> {
        let state = self.get_state();
        let turn = match state {
            State::Turn(m) => m,
            _ => {
                return None;
            }
        };

        if i > 8 {
            return None;
        }

        match self.marks[i] {
            Some(_) => None,
            None => {
                self.marks[i] = Some(turn);
                Some(())
            }
        }
    }
    fn empty(&self) -> Vec<usize> {
        self.marks
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_none())
            .map(|(i, _)| i)
            .collect()
    }
}

fn minimax(board: Board, player: Mark) -> (usize, i8) {
    let possible = board.empty();
    let mut results = Vec::new();
    for mve in possible {
        let mut new_board = board;
        new_board.place(mve);
        match new_board.get_state() {
            State::Turn(_) => {
                results.push((mve, -minimax(new_board, player.other()).1));
            }
            State::Tie => results.push((mve, 0)),
            State::Winner(m) => results.push((mve, m.to_value() * player.to_value())),
        }
    }
    *results.iter().max_by_key(|t| t.1).unwrap()
}

pub struct TicTacToe {
    players: HashMap<String, Board>,
}

impl ChatCommand for TicTacToe {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }
    fn help(&self) -> String {
        "usage: !tictactoe/!ttt (1-9/reset)".to_owned()
    }
    fn names() -> Vec<String> {
        vec!["tictactoe".to_owned(), "ttt".to_owned()]
    }
    fn handle(
        &mut self,
        api: &mut crate::api::TwitchApiWrapper,
        ctx: &twitcheventsub::MessageData,
    ) -> anyhow::Result<()> {
        let arg = ctx.message.text.split_whitespace().nth(1);
        match arg {
            None => {
                let _ = api.send_chat_message(self.help());
                return Ok(());
            }
            Some("reset") => {
                self.players.remove(&ctx.chatter.id);
            }
            _ => {
                if let Some(arg) = arg {
                    match arg.chars().next() {
                        Some(c @ '1'..'9') => {
                            if !self.players.contains_key(&ctx.chatter.id) {
                                self.players.insert(ctx.chatter.id.clone(), Board::new());
                            }
                            let board = self.players.get_mut(&ctx.chatter.id).unwrap();
                            board.place((c.to_digit(10).unwrap() - 1) as usize);
                            let bot_move = minimax(*board, board.get_state().to_mark()).0;
                            board.place(bot_move);
                        }
                        _ => {
                            let _ = api.send_chat_message(self.help());
                            return Ok(());
                        }
                    }
                }
                let _ = api.send_chat_message(self.help());
                return Ok(());
            }
        }
        Ok(())
    }
}

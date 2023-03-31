use serde::{Deserialize, Serialize};
use tokio::sync::watch;

#[derive(Debug)]
pub struct Game {
    pub id: String,
    pub state: State,
    pub state_changes: watch::Sender<State>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct State {
    pub players: Vec<String>,
    pub board: Vec<char>,
    pub chat: Vec<(usize, String)>,
}

impl State {
    pub fn new() -> State {
        State {
            players: Vec::new(),
            board: vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            chat: Vec::new(),
        }
    }
}

impl Game {
    pub fn new(id: String, state_changes: watch::Sender<State>) -> Game {
        let game = Game {
            id: id,
            state: State::new(),
            state_changes: state_changes,
        };

        return game;
    }

    pub fn handle_msg(&mut self, msg: StateChange) -> Result<(), String> {
        println!("Game: Handle Msg: {:?}", msg);
        match msg {
            StateChange::ChatMsg(chat) => {
                let id = self.state.chat.len();
                self.state.chat.push((id, chat.clone()));
                self.state_changes.send(self.state.clone()).unwrap();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateChange {
    ChatMsg(String),
}

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
    pub fn new(id: String) -> (Game, watch::Receiver<State>) {
        let state = State::new();
        let (tx, rx) = watch::channel(state.clone());

        let game = Game {
            id: id,
            state: state,
            state_changes: tx,
        };

        return (game, rx);
    }

    pub fn handle_msg(&mut self, msg: &FromBrowser) -> Result<(), String> {
        println!("Game: Handle Msg: {:?}", msg);
        match msg {
            FromBrowser::ChatMsg(msg_text) => {
                let id = self.state.chat.len();
                self.state.chat.push((id, msg_text.clone()));
                self.state_changes.send(self.state.clone()).unwrap();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum FromBrowser {
    ChatMsg(String),
}

#[derive(Debug, Clone, Serialize)]
pub enum ToBrowser {
    JoinedGame { token: String, state: State },
    GameState(State),
}

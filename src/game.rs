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
    pub players: Vec<Player>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Player {
    pub id: usize,
    pub team: char,
    pub name: String,
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

    pub fn add_player(&mut self, name: String) -> Result<Player, String> {
        if self.state.players.len() >= 2 {
            return Err("Game is full".to_string());
        }

        let team = if self.state.players.len() == 0 {
            'X'
        } else {
            'O'
        };

        let player = Player {
            id: self.state.players.len(),
            team: team,
            name: name,
        };
        self.state.players.push(player.clone());
        self.state_changes.send_replace(self.state.clone());
        Ok(player)
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
            FromBrowser::Move { pos, player } => {
                if self.state.board[*pos] != ' ' {
                    return Err("Invalid move".to_string());
                }

                self.state.board[*pos] = self.state.players[*player].team;
                self.state_changes.send(self.state.clone()).unwrap();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum FromBrowser {
    ChatMsg(String),
    Move { pos: usize, player: usize },
}

#[derive(Debug, Clone, Serialize)]
pub enum ToBrowser {
    JoinedGame {
        token: String,
        team: char,
        state: State,
    },
    GameState(State),
}

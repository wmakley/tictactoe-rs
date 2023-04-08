use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tracing::debug;

#[derive(Debug)]
pub struct Game {
    pub id: String,
    pub state: State,
    pub state_changes: watch::Sender<State>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct State {
    pub turn: char,
    pub winner: Option<EndState>,
    pub players: Vec<Player>,
    pub board: Vec<char>,
    pub chat: Vec<ChatMessage>,
}

impl State {
    pub fn new() -> State {
        State {
            turn: 'X',
            winner: None,
            players: Vec::new(),
            board: vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            chat: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EndState {
    Win(char),
    Draw,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Player {
    pub team: char,
    pub name: String,
    pub wins: i32,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.team)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub id: usize,
    pub source: ChatMessageSource,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChatMessageSource {
    Player(char),
    System,
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
            match self.state.players[0].team {
                'X' => 'O',
                'O' => 'X',

                _ => unreachable!(),
            }
        };

        let player = Player {
            team: team,
            name: name,
            wins: 0,
        };
        self.state.players.push(player.clone());
        self.add_chat_message(
            ChatMessageSource::System,
            format!("{} ({}) has joined the game", player.name, player.team),
        )
        .unwrap();
        Ok(player)
    }

    fn add_chat_message(&mut self, source: ChatMessageSource, text: String) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("Empty message".to_string());
        }

        let id = self.state.chat.len();
        self.state.chat.push(ChatMessage {
            id: id,
            source: source,
            text: text,
        });
        Ok(())
    }

    pub fn get_player(&self, team: char) -> Option<&Player> {
        self.state.players.iter().find(|p| p.team == team)
    }

    pub fn get_player_mut(&mut self, team: char) -> Option<&mut Player> {
        self.state.players.iter_mut().find(|p| p.team == team)
    }

    pub fn remove_player(&mut self, team: char) {
        let player = match self.state.players.iter().find(|p| p.team == team) {
            Some(p) => p,
            None => return,
        };
        self.add_chat_message(
            ChatMessageSource::System,
            format!("{} has left the game", player.name),
        )
        .unwrap();
        self.state.players.retain(|p| p.team != team);
    }

    pub fn take_turn(&mut self, player: char, space: usize) -> Result<(), String> {
        if self.state.players.len() < 2 {
            return Err("Not enough players".to_string());
        }

        if self.state.winner.is_some() {
            return Err("Game is over".to_string());
        }

        if self.state.turn != player {
            return Err("Not your turn".to_string());
        }

        if self.state.board[space] != ' ' {
            return Err("Invalid move".to_string());
        }

        self.state.board[space] = player;
        self.state.turn = if self.state.turn == 'X' { 'O' } else { 'X' };

        self.add_chat_message(
            ChatMessageSource::Player(player),
            format!(
                "Played {} at ({}, {}).",
                player,
                space % 3 + 1,
                space / 3 + 1
            ),
        )
        .unwrap();

        if let Some(winner) = self.check_for_win() {
            self.state.winner = Some(EndState::Win(winner));
            self.get_player_mut(winner).unwrap().wins += 1;
            self.add_chat_message(
                ChatMessageSource::System,
                format!("{} wins!", self.get_player(winner).unwrap()),
            )
            .unwrap();
        } else if self.check_for_draw() {
            self.state.winner = Some(EndState::Draw);
            self.add_chat_message(ChatMessageSource::System, "It's a draw!".to_string())
                .unwrap();
        }

        Ok(())
    }

    pub fn broadcast_state(&self) {
        self.state_changes.send_replace(self.state.clone());
    }

    fn check_for_win(&self) -> Option<char> {
        let winning_combos = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        for combo in winning_combos.iter() {
            let mut winner = self.state.board[combo[0]];
            if winner == ' ' {
                continue;
            }

            for i in 1..3 {
                if self.state.board[combo[i]] != winner {
                    winner = ' ';
                    break;
                }
            }

            if winner != ' ' {
                return Some(winner);
            }
        }

        None
    }

    fn check_for_draw(&self) -> bool {
        self.state.board.iter().all(|&c| c != ' ')
    }

    pub fn handle_msg(&mut self, player: char, msg: FromBrowser) -> Result<bool, String> {
        debug!("Game: Handle Msg: {:?}", msg);
        match msg {
            FromBrowser::ChatMsg { text } => {
                self.add_chat_message(ChatMessageSource::Player(player), text)?;
            }
            FromBrowser::Move { space } => self.take_turn(player, space)?,
            FromBrowser::Rematch => {
                self.add_chat_message(ChatMessageSource::Player(player), "Rematch!".to_string())
                    .unwrap();
                self.state.board.iter_mut().for_each(|c| *c = ' ');
                self.state.turn = 'X';
                self.state.winner = None;
            }
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum FromBrowser {
    ChatMsg { text: String },
    Move { space: usize },
    Rematch,
}

#[derive(Debug, Clone, Serialize)]
pub enum ToBrowser {
    JoinedGame {
        token: String,
        team: char,
        state: State,
    },
    GameState(State),
    Error(String),
}

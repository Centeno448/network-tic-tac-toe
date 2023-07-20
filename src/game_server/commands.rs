use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandCategory {
    PlayerConnected,
    PlayerDisconnected,
    GameStart,
    EndTurn,
    GameOver,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commmand {
    pub category: CommandCategory,
    pub body: String,
}

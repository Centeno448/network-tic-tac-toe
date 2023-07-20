use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum CommandCategory {
    PlayerConnected,
    PlayerDisconnected,
    GameStart,
    EndTurn,
    GameOver,
}

#[derive(Debug, Serialize)]
pub struct Commmand<S: Serialize> {
    pub category: CommandCategory,
    pub body: S,
}

impl<S: Serialize> Commmand<S> {
    pub fn new(category: CommandCategory, body: S) -> Self {
        Commmand { category, body }
    }
}

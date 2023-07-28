use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum CommandCategory {
    Connected,
    PlayerConnected,
    PlayerDisconnected,
    GameStart,
    Turn,
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

    pub fn new_serialized(category: CommandCategory, body: S) -> String {
        let command = Self::new(category, body);

        serde_json::to_string(&command).unwrap_or("".into())
    }
}

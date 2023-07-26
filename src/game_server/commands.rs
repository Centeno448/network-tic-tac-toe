use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::{Actor, Message};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum CommandCategory {
    PlayerConnected,
    PlayerDisconnected,
    GameStart,
    Turn,
    GameOver,
}

#[derive(Debug, Serialize)]
pub enum TurnMove {
    LL,
    ML,
    UL,
    LM,
    MM,
    UM,
    LR,
    MR,
    UR,
    None,
}

impl From<&str> for TurnMove {
    fn from(value: &str) -> Self {
        match value {
            "LL" => Self::LL,
            "ML" => Self::ML,
            "UL" => Self::UL,
            "LM" => Self::LM,
            "MM" => Self::MM,
            "UM" => Self::UM,
            "LR" => Self::LR,
            "MR" => Self::MR,
            "UR" => Self::UR,
            _ => Self::None,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TeamSymbol {
    Cross,
    Circle,
}

impl<A, M> MessageResponse<A, M> for TeamSymbol
where
    A: Actor,
    M: Message<Result = TeamSymbol>,
{
    fn handle(self, _: &mut A::Context, tx: Option<OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
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

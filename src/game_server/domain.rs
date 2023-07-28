use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::{Actor, Message};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
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

impl std::fmt::Display for TurnMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::LL => write!(f, "LL"),
            Self::ML => write!(f, "ML"),
            Self::UL => write!(f, "UL"),
            Self::LM => write!(f, "LM"),
            Self::MM => write!(f, "MM"),
            Self::UM => write!(f, "UM"),
            Self::LR => write!(f, "LR"),
            Self::MR => write!(f, "MR"),
            Self::UR => write!(f, "UR"),
            Self::None => write!(f, ""),
        }
    }
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

#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
pub enum TeamSymbol {
    Cross,
    Circle,
}

impl std::fmt::Display for TeamSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Cross => write!(f, "Cross"),
            Self::Circle => write!(f, "Circle"),
        }
    }
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

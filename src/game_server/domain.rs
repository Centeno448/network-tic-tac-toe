use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::{Actor, Message};
use serde::Serialize;
use uuid::Uuid;

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

pub struct RoomResponse(pub Option<Uuid>);

impl<A, M> MessageResponse<A, M> for RoomResponse
where
    A: Actor,
    M: Message<Result = RoomResponse>,
{
    fn handle(self, _: &mut A::Context, tx: Option<OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
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

#[cfg(test)]
mod tests {
    use super::TeamSymbol;
    use super::TurnMove;

    #[test]
    pub fn turn_move_deserializes_correctly() {
        assert_eq!(Into::<TurnMove>::into("LL"), TurnMove::LL);
        assert_eq!(Into::<TurnMove>::into("ML"), TurnMove::ML);
        assert_eq!(Into::<TurnMove>::into("UL"), TurnMove::UL);
        assert_eq!(Into::<TurnMove>::into("LM"), TurnMove::LM);
        assert_eq!(Into::<TurnMove>::into("MM"), TurnMove::MM);
        assert_eq!(Into::<TurnMove>::into("UM"), TurnMove::UM);
        assert_eq!(Into::<TurnMove>::into("LR"), TurnMove::LR);
        assert_eq!(Into::<TurnMove>::into("MR"), TurnMove::MR);
        assert_eq!(Into::<TurnMove>::into("UR"), TurnMove::UR);
        assert_eq!(Into::<TurnMove>::into("other"), TurnMove::None);
    }

    #[test]
    pub fn turn_move_displays_correctly() {
        assert_eq!(format!("{}", TurnMove::LL), "LL");
        assert_eq!(format!("{}", TurnMove::ML), "ML");
        assert_eq!(format!("{}", TurnMove::UL), "UL");
        assert_eq!(format!("{}", TurnMove::LM), "LM");
        assert_eq!(format!("{}", TurnMove::MM), "MM");
        assert_eq!(format!("{}", TurnMove::UM), "UM");
        assert_eq!(format!("{}", TurnMove::LR), "LR");
        assert_eq!(format!("{}", TurnMove::MR), "MR");
        assert_eq!(format!("{}", TurnMove::UR), "UR");
        assert_eq!(format!("{}", TurnMove::None), "");
    }

    #[test]
    pub fn team_symbol_displays_correctly() {
        assert_eq!(format!("{}", TeamSymbol::Circle), "Circle");
        assert_eq!(format!("{}", TeamSymbol::Cross), "Cross");
    }
}

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "message", content = "content")]
pub enum PlayerMessage {
    Start,
    List,
    Leave,
    Create(String),
    Join(Uuid),
    Turn(String),
    Username(String),
}

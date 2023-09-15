use uuid::Uuid;

use crate::game_server::{domain::TeamSymbol, GameRoom, GameRoomStatus};

pub enum ShouldDeleteRoom {
    Yes,
    No,
}

pub fn remove_player_from_room(room: &mut GameRoom, player_id: &Uuid) -> ShouldDeleteRoom {
    room.players.remove(player_id);
    if room.players.len() > 0 {
        reset_room(room);
        return ShouldDeleteRoom::No;
    } else {
        return ShouldDeleteRoom::Yes;
    }
}

fn reset_room(room: &mut GameRoom) {
    room.moves_made.clear();
    room.status = GameRoomStatus::Waiting;
    room.current_turn = TeamSymbol::Cross;
}

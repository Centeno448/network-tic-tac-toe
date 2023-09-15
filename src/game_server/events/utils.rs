use uuid::Uuid;

use crate::game_server::{
    domain::TeamSymbol, CommandCategory, Commmand, GameRoom, GameRoomStatus, GameServer,
};

pub enum ShouldDeleteRoom {
    Yes,
    No,
}

pub fn handle_potential_room_deletion(
    should_delete_room: ShouldDeleteRoom,
    server: &mut GameServer,
    player_id: &Uuid,
    room_id: &Uuid,
) {
    match should_delete_room {
        ShouldDeleteRoom::No => {
            let command = Commmand::new(CommandCategory::PlayerLeft, "".to_string());
            let result = serde_json::to_string(&command).unwrap_or("".into());

            server.send_message(room_id, &result, player_id.clone());
        }
        ShouldDeleteRoom::Yes => {
            server.rooms.remove(room_id);
        }
    }
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

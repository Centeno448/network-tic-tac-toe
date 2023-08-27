use crate::helpers::{process_message, send_message, spawn_app, MatchListResponse};

#[actix_web::test]
async fn when_no_matches_exists_returns_empty_array() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects

    send_message(&mut player_one, "/list_matches").await;

    let player_one_response = process_message(&mut player_one).await; // Player 3 recieves match list

    let response: MatchListResponse =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(response.category, "MatchList");
    assert_eq!(response.body.matches.len(), 0);
}

#[actix_web::test]
async fn when_matches_exists_returns_them() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects
    process_message(&mut player_three).await; // Player 3 connects

    send_message(&mut player_one, "/create_match player-1-room").await;
    send_message(&mut player_two, "/create_match player-2-room").await;

    process_message(&mut player_one).await; // Player 1 recieves match creation confirmation
    process_message(&mut player_two).await; // Player 2 recieves match creation confirmation

    send_message(&mut player_three, "/list_matches").await;

    let player_three_response = process_message(&mut player_three).await; // Player 3 recieves match list

    let response: MatchListResponse =
        serde_json::from_str(player_three_response.to_text().unwrap()).unwrap();

    assert_eq!(response.category, "MatchList");
    assert_eq!(response.body.matches.len(), 2);
    assert!(response
        .body
        .matches
        .iter()
        .find(|m| m.room_name == "player-1-room")
        .is_some());
    assert!(response
        .body
        .matches
        .iter()
        .find(|m| m.room_name == "player-2-room")
        .is_some());
}

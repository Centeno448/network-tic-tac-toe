use crate::helpers::{
    build_create_message, build_join_message, process_message, process_message_result,
    send_message, spawn_app, MatchListResponse, LIST_MESSAGE, START_MESSAGE,
};

#[actix_web::test]
async fn circle_player_cant_start_game() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects

    send_message(&mut player_one, &build_create_message("room")).await;

    process_message(&mut player_one).await;

    send_message(&mut player_two, LIST_MESSAGE).await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response.body.matches.first().unwrap().match_id;

    send_message(&mut player_two, &build_join_message(match_id)).await;

    process_message(&mut player_two).await;
    process_message(&mut player_one).await;

    send_message(&mut player_two, START_MESSAGE).await;

    let player_one_response = process_message_result(&mut player_one).await;
    let player_two_response = process_message_result(&mut player_two).await;

    assert!(player_one_response.is_none());
    assert!(player_two_response.is_none());
}

#[actix_web::test]
async fn cross_player_can_start_game() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects

    send_message(&mut player_one, &build_create_message("room")).await;

    process_message(&mut player_one).await;

    send_message(&mut player_two, LIST_MESSAGE).await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response.body.matches.first().unwrap().match_id;

    send_message(&mut player_two, &build_join_message(match_id)).await;

    process_message(&mut player_two).await;
    process_message(&mut player_one).await;

    send_message(&mut player_one, START_MESSAGE).await;

    let player_one_response = process_message_result(&mut player_one).await;
    let player_two_response = process_message_result(&mut player_two).await;

    assert!(player_one_response.is_some());
    assert!(player_two_response.is_some());

    let player_one_response = player_one_response
        .unwrap()
        .expect("Failed to recieve message.");
    let player_two_response = player_two_response
        .unwrap()
        .expect("Failed to recieve message.");

    let expected = serde_json::json!({
        "category": "GameStart",
        "body": ""
    });

    let player_one_response: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();
    let player_two_response: serde_json::Value =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    assert_eq!(player_one_response, expected);
    assert_eq!(player_two_response, expected);
}

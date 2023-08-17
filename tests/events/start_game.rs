use crate::helpers::{process_message, process_message_result, send_message, spawn_app};

#[actix_web::test]
async fn circle_player_cant_start_game() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects
    process_message(&mut player_one).await; // Player 1 recieves confirmation player 2 connected

    send_message(&mut player_two, "/start").await;

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
    process_message(&mut player_one).await; // Player 1 recieves confirmation player 2 connected

    send_message(&mut player_one, "/start").await;

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

    assert!(player_one_response.is_text());
    assert!(player_two_response.is_text());

    assert!(player_one_response.to_text().unwrap().contains("GameStart"));
    assert!(player_two_response.to_text().unwrap().contains("GameStart"));
}

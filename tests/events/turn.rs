use crate::helpers::*;

#[actix_web::test]
async fn server_ignores_invalid_turn() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects
    process_message(&mut player_one).await; // Player 1 recieves confirmation player 2 connected

    send_message(&mut player_one, "/start").await; // Game start

    process_message(&mut player_one).await; // Player 1 recieves game start
    process_message(&mut player_two).await; // Player 2 recieves game start

    send_message(&mut player_two, "/turn MM").await; // Invalid turn

    let player_one_response = process_message_result(&mut player_one).await;

    assert!(
        player_one_response.is_none(),
        "Invalid turn is not notified to player one"
    );
}

#[actix_web::test]
async fn server_ignores_duplicate_turn() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects
    process_message(&mut player_one).await; // Player 1 recieves confirmation player 2 connected

    send_message(&mut player_one, "/start").await; // Game start

    process_message(&mut player_one).await; // Player 1 recieves game start
    process_message(&mut player_two).await; // Player 2 recieves game start

    send_message(&mut player_one, "/turn MM").await; // Player 1 turn

    process_message(&mut player_two).await; // Player 2 recieves turn

    send_message(&mut player_one, "/turn MM").await; // Player 2 duplicate turn

    let player_one_response = process_message_result(&mut player_one).await;

    assert!(
        player_one_response.is_none(),
        "Duplicate turn is not notified to player one"
    );
}

#[actix_web::test]
async fn server_processes_valid_turn() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects
    process_message(&mut player_one).await; // Player 1 recieves confirmation player 2 connected

    send_message(&mut player_one, "/start").await; // Game start

    process_message(&mut player_one).await; // Player 1 recieves game start
    process_message(&mut player_two).await; // Player 2 recieves game start

    send_message(&mut player_one, "/turn MM").await; // Player 1 turn

    let player_two_msg = process_message_result(&mut player_two).await; // Player 2 recieves turn

    send_message(&mut player_two, "/turn LL").await; // Player 2 turn

    let player_one_msg = process_message_result(&mut player_one).await; // Player 1 recieves turn

    assert!(player_two_msg.is_some());
    assert!(player_one_msg.is_some());

    let player_two_msg = player_two_msg.unwrap().expect("Failed to recieve message");
    let player_one_msg = player_one_msg.unwrap().expect("Failed to recieve message");

    let expected_player_two_msg = serde_json::json!({
        "category": "Turn",
        "body": "MM"
    });

    let expected_player_one_msg = serde_json::json!({
        "category": "Turn",
        "body": "LL"
    });

    let player_one_msg: serde_json::Value =
        serde_json::from_str(player_one_msg.to_text().unwrap()).unwrap();
    let player_two_msg: serde_json::Value =
        serde_json::from_str(player_two_msg.to_text().unwrap()).unwrap();

    assert_eq!(
        player_two_msg, expected_player_two_msg,
        "Player one turn is notified to player two"
    );
    assert_eq!(
        player_one_msg, expected_player_one_msg,
        "Player two turn is notified to player one"
    );
}

#[actix_web::test]
async fn game_ends_on_tie() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game_for_tie(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, "/turn ML").await; // Final turn
    process_message(&mut player_two).await; // Player 2 recieves final turn

    let player_one_msg = process_message(&mut player_one).await;
    let player_two_msg = process_message(&mut player_two).await;

    let expected = serde_json::json!({
        "category": "GameOver",
        "body": {
            "outcome": "tie"
        }
    });

    let player_one_msg: serde_json::Value =
        serde_json::from_str(player_one_msg.to_text().unwrap()).unwrap();
    let player_two_msg: serde_json::Value =
        serde_json::from_str(player_two_msg.to_text().unwrap()).unwrap();

    assert_eq!(player_one_msg, expected);
    assert_eq!(player_two_msg, expected);
}

#[actix_web::test]
async fn game_ends_on_diagonal_victory() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game_for_diagonal_victory(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, "/turn UR").await; // Final turn
    process_message(&mut player_two).await; // Player 2 recieves final turn

    let player_one_msg = process_message(&mut player_one).await;
    let player_two_msg = process_message(&mut player_two).await;

    let expected = serde_json::json!({
        "category": "GameOver",
        "body": {
            "outcome": "victory",
            "winner": "Cross"
        }
    });

    let player_one_msg: serde_json::Value =
        serde_json::from_str(player_one_msg.to_text().unwrap()).unwrap();
    let player_two_msg: serde_json::Value =
        serde_json::from_str(player_two_msg.to_text().unwrap()).unwrap();

    assert_eq!(player_one_msg, expected);
    assert_eq!(player_two_msg, expected);
}

#[actix_web::test]
async fn game_ends_on_cross_victory() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game_for_cross_victory(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, "/turn LR").await; // Final turn
    process_message(&mut player_two).await; // Player 2 recieves final turn

    let player_one_msg = process_message(&mut player_one).await;
    let player_two_msg = process_message(&mut player_two).await;

    let expected = serde_json::json!({
        "category": "GameOver",
        "body": {
            "outcome": "victory",
            "winner": "Cross"
        }
    });

    let player_one_msg: serde_json::Value =
        serde_json::from_str(player_one_msg.to_text().unwrap()).unwrap();
    let player_two_msg: serde_json::Value =
        serde_json::from_str(player_two_msg.to_text().unwrap()).unwrap();

    assert_eq!(player_one_msg, expected);
    assert_eq!(player_two_msg, expected);
}

#[actix_web::test]
async fn game_ends_on_circle_victory() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game_for_circle_victory(&mut player_one, &mut player_two).await;

    send_message(&mut player_two, "/turn UR").await; // Final turn
    process_message(&mut player_one).await; // Player 1 recieves final turn

    let player_one_msg = process_message(&mut player_one).await;
    let player_two_msg = process_message(&mut player_two).await;

    let expected = serde_json::json!({
        "category": "GameOver",
        "body": {
            "outcome": "victory",
            "winner": "Circle"
        }
    });

    let player_one_msg: serde_json::Value =
        serde_json::from_str(player_one_msg.to_text().unwrap()).unwrap();
    let player_two_msg: serde_json::Value =
        serde_json::from_str(player_two_msg.to_text().unwrap()).unwrap();

    assert_eq!(player_one_msg, expected);
    assert_eq!(player_two_msg, expected);
}

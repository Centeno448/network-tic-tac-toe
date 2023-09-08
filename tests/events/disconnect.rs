use crate::helpers::*;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

#[actix_web::test]
async fn player_can_disconnect_before_game_starts() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game(&mut player_one, &mut player_two).await;

    let _ = player_two
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    let player_one_response = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "PlayerDisconnected",
        "body": "",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn player_can_disconnect_during_ongoing_match() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_and_start_game(&mut player_one, &mut player_two).await;

    let _ = player_two
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    let player_one_response = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "PlayerDisconnected",
        "body": "",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn when_player_disconnects_during_match_resets_room() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, "/turn MM").await;

    process_message(&mut player_two).await;

    let _ = player_two
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    process_message(&mut player_one).await;

    join_room(&mut player_one, &mut player_three).await; // Player 3 joins player 1's room.

    send_message(&mut player_one, "/start").await;

    process_message(&mut player_one).await;
    process_message(&mut player_three).await;

    send_message(&mut player_one, "/turn MM").await; // Duplicate turn if room had not reset.

    let player_three_response = process_message(&mut player_three).await;

    let expected = serde_json::json!({
        "category": "Turn",
        "body": "MM",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_three_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn when_cross_player_disconnects_circle_player_becomes_cross() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, "/turn MM").await;

    process_message(&mut player_two).await;

    let _ = player_one
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    process_message(&mut player_two).await;

    join_room(&mut player_two, &mut player_three).await; // Player 3 joins player 1's room.

    send_message(&mut player_two, "/start").await;

    process_message(&mut player_two).await;
    process_message(&mut player_three).await;

    send_message(&mut player_two, "/turn MM").await; // Duplicate turn if room had not reset.

    let player_three_response = process_message(&mut player_three).await;

    let expected = serde_json::json!({
        "category": "Turn",
        "body": "MM",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_three_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn when_both_players_disconnect_deletes_room() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    let _ = player_two
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    process_message(&mut player_one).await;

    let _ = player_one
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await;

    send_message(&mut player_three, "/list").await;

    let player_three_response = process_message(&mut player_three).await;
    let player_three_response: MatchListResponse =
        serde_json::from_str(player_three_response.to_text().unwrap()).unwrap();

    assert_eq!(player_three_response.body.matches.len(), 0);
}

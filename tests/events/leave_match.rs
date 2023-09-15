use crate::helpers::*;

#[actix_web::test]
async fn player_can_leave_before_game_starts() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_two, LEAVE_MESSAGE).await;

    let player_one_response = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "PlayerLeft",
        "body": "",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn player_can_leave_during_ongoing_match() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_two, LEAVE_MESSAGE).await;

    let player_one_response = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "PlayerLeft",
        "body": "",
    });
    let result: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn when_player_leaves_during_match_resets_room() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, &build_turn_message("MM")).await;

    process_message(&mut player_two).await;

    send_message(&mut player_two, LEAVE_MESSAGE).await;

    process_message(&mut player_one).await;

    join_room(&mut player_one, &mut player_three).await; // Player 3 joins player 1's room.

    send_message(&mut player_one, START_MESSAGE).await;

    process_message(&mut player_one).await;
    process_message(&mut player_three).await;

    send_message(&mut player_one, &build_turn_message("MM")).await; // Duplicate turn if room had not reset.

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
async fn when_cross_player_leaves_circle_player_becomes_cross() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_one, &build_turn_message("MM")).await;

    process_message(&mut player_two).await;

    send_message(&mut player_one, LEAVE_MESSAGE).await; // Cross player leaves

    process_message(&mut player_two).await;

    join_room(&mut player_two, &mut player_three).await; // Player 3 joins player 1's room.

    send_message(&mut player_two, START_MESSAGE).await;

    process_message(&mut player_two).await;
    process_message(&mut player_three).await;

    send_message(&mut player_two, &build_turn_message("MM")).await; // Duplicate turn if room had not reset.

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
async fn when_both_players_leave_deletes_room() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    process_message(&mut player_three).await; // Player 3 connects

    setup_and_start_game(&mut player_one, &mut player_two).await;

    send_message(&mut player_two, LEAVE_MESSAGE).await;

    process_message(&mut player_one).await;

    send_message(&mut player_one, LEAVE_MESSAGE).await;

    send_message(&mut player_three, LIST_MESSAGE).await;

    let player_three_response = process_message(&mut player_three).await;
    let player_three_response: MatchListResponse =
        serde_json::from_str(player_three_response.to_text().unwrap()).unwrap();

    assert_eq!(player_three_response.body.matches.len(), 0);
}

#[actix_web::test]
async fn when_player_leaves_they_can_join_another_match() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    setup_and_start_game(&mut player_one, &mut player_two).await; // Player 1 and 2 are in a match

    process_message(&mut player_three).await; // Player 3 connects

    send_message(&mut player_three, &build_username_message("playerthree")).await; // Set player three username

    send_message(&mut player_three, &build_create_message("player-3-room")).await;

    send_message(&mut player_two, LEAVE_MESSAGE).await; // Player 2 leaves

    send_message(&mut player_two, LIST_MESSAGE).await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response
        .body
        .matches
        .iter()
        .filter(|m| m.room_name == "player-3-room")
        .take(1)
        .next()
        .unwrap()
        .match_id;

    send_message(&mut player_two, &build_join_message(match_id)).await;

    let player_two_response = process_message(&mut player_two).await;

    let expected_p2_response = serde_json::json!({
        "category": "MatchJoined",
        "body": "playerthree"
    });

    let player_two_response: serde_json::Value =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    assert_eq!(player_two_response, expected_p2_response);
}

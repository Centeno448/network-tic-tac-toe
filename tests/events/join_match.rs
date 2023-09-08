use crate::helpers::{process_message, send_message, spawn_app, MatchListResponse};

#[actix_web::test]
async fn existing_match_can_be_joined() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects

    send_message(&mut player_one, "/username playerone").await; // Set player one username
    send_message(&mut player_two, "/username playertwo").await; // Set player two username

    send_message(&mut player_one, "/create my-own-room").await;

    process_message(&mut player_one).await;

    send_message(&mut player_two, "/list").await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response.body.matches.first().unwrap().match_id;

    send_message(&mut player_two, &format!("/join {}", match_id)).await;

    let player_one_response = process_message(&mut player_one).await;
    let player_one_response: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    let expected_p1_response = serde_json::json!({
        "category": "PlayerConnected",
        "body": "playertwo"
    });

    let player_two_response = process_message(&mut player_two).await;

    let expected_p2_response = serde_json::json!({
        "category": "MatchJoined",
        "body": "playerone"
    });

    let player_two_response: serde_json::Value =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    assert_eq!(player_two_response, expected_p2_response);
    assert_eq!(player_one_response, expected_p1_response);
}

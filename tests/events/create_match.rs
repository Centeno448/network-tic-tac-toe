use crate::helpers::{process_message, send_message, spawn_app};

#[actix_web::test]
async fn match_can_be_created() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects

    send_message(&mut player_one, "/create my-own-room").await;

    let player_one_response = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "MatchCreated",
        "body": "my-own-room"
    });

    let player_one_response: serde_json::Value =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(player_one_response, expected);
}

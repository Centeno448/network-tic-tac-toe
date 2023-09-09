use crate::helpers::{build_create_message, process_message, send_message, spawn_app};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CreateMatchResponse {
    category: String,
    body: String,
}

#[actix_web::test]
async fn match_can_be_created() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;

    process_message(&mut player_one).await; // Player 1 connects

    send_message(&mut player_one, &build_create_message("my cool room")).await;

    let player_one_response = process_message(&mut player_one).await;

    let player_one_response: CreateMatchResponse =
        serde_json::from_str(player_one_response.to_text().unwrap()).unwrap();

    assert_eq!(player_one_response.category, "MatchCreated");
    assert!(Uuid::try_parse(&player_one_response.body).is_ok());
}

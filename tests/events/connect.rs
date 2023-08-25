use crate::helpers::{process_message, process_message_result, spawn_app};

#[actix_web::test]
async fn when_player_connects_they_recieve_confirmation() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;

    let msg = process_message(&mut player_one).await;

    let expected = serde_json::json!({
        "category": "Connected",
        "body": "",
    });
    let result: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[actix_web::test]
async fn when_another_player_connects_others_are_not_notified() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;
    let mut player_three = test_app.connect_player().await;

    let _ = process_message(&mut player_one).await;
    let _ = process_message(&mut player_two).await;
    let _ = process_message(&mut player_three).await;
    let player_one_msg = process_message_result(&mut player_one).await;
    let player_two_msg = process_message_result(&mut player_two).await;

    assert!(player_one_msg.is_none());
    assert!(player_two_msg.is_none());
}

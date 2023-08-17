use crate::helpers::{process_message, spawn_app};

#[actix_web::test]
async fn when_first_player_connects_they_are_assigned_cross() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;

    let msg = process_message(&mut player_one).await;

    assert!(msg.is_text());
    assert!(msg.to_text().unwrap().contains("Cross"));
}

#[actix_web::test]
async fn when_second_player_connects_they_are_assigned_circle() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    let _ = process_message(&mut player_one).await;
    let msg = process_message(&mut player_two).await;

    assert!(msg.is_text());
    assert!(msg.to_text().unwrap().contains("Circle"));
}

#[actix_web::test]
async fn when_second_player_connects_player_one_is_notified() {
    let test_app = spawn_app().await;

    let mut player_one = test_app.connect_player().await;
    let mut player_two = test_app.connect_player().await;

    let _ = process_message(&mut player_one).await;
    let _ = process_message(&mut player_two).await;
    let msg = process_message(&mut player_one).await;

    assert!(msg.is_text());
    assert!(msg.to_text().unwrap().contains("PlayerConnected"));
}

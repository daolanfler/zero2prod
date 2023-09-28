use crate::helpers::spawn_app;


#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    println!("res status is {:?}", response.status());

    // 注意这里命令行不能开代理，否则是 502 奇怪
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
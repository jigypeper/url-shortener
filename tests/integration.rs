use anyhow::Result;
use reqwest::{StatusCode, redirect::Policy};
use serde_json::json;
use serial_test::serial;
use std::net::TcpListener;
use url_shortener::state::State;

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let database = {
        let mut config = tokio_postgres::Config::new();
        config.host("localhost");
        config.port(5432);
        config.dbname("test_url_shortener");
        config.user("postgres");
        config.password("postgres");

        let mgr = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
        deadpool_postgres::Pool::builder(mgr)
            .runtime(deadpool_postgres::Runtime::Tokio1)
            .build()
            .expect("Failed to create database pool")
    };

    // Clear the database before each test
    let client = database.get().await.expect("Failed to get client");
    client
        .execute("TRUNCATE TABLE link", &[])
        .await
        .expect("Failed to truncate table");

    let state = State::new(database);
    let server = url_shortener::api::listen(listener, state).expect("Failed to bind address");
    tokio::spawn(server);

    address
}

#[tokio::test]
#[serial]
async fn test_create_and_get_link() -> Result<()> {
    let app_address = spawn_app().await;
    // need to stop the redirect so we can ensure status code is correct.
    let client = reqwest::Client::builder().redirect(Policy::none()).build()?;
    // Create a new short URL
    let response = client
        .post(&format!("{}/api/links", app_address))
        .json(&json!({
            "url": "https://www.example.com"
        }))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::CREATED);

    let json = response.json::<serde_json::Value>().await?;
    let id = json["id"].as_str().unwrap();
    assert_eq!(id.len(), 8);
    assert_eq!(json["url"], "https://www.example.com");

    // Try to get the URL
    let response = client.get(&format!("{}/api/{}", app_address, id)).send().await?;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()["location"], "https://www.example.com");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_delete_link() -> Result<()> {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    // First create a link
    let response = client
        .post(&format!("{}/api/links", app_address))
        .json(&json!({
            "url": "https://www.example.com"
        }))
        .send()
        .await?;

    let json = response.json::<serde_json::Value>().await?;
    let id = json["id"].as_str().unwrap();

    // Delete the link
    let response = client.delete(&format!("{}/api/links/{}", app_address, id)).send().await?;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let response = client.get(&format!("{}/{}", app_address, id)).send().await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_not_found() -> Result<()> {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/nonexistent", app_address)).send().await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

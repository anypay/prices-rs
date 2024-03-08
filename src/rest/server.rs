mod rest;

use axum::{
    routing::{get, post},
    Json, Router,
};
use models::{Price, PriceCreateRequest, PriceHistory, PriceSource};
use std::net::SocketAddr;
use uuid::Uuid;
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use dotenv::dotenv;

async fn list_prices() -> Json<Vec<Price>> {
    Json(vec![Price {
        id: Uuid::new_v4(),
        base_currency: "USD".to_string(),
        currency: "EUR".to_string(),
        value: 1.1,
        source: "Example Source".to_string(),
        created_at: "2022-01-01T00:00:00Z".to_string(),
        updated_at: "2022-01-02T00:00:00Z".to_string(),
    }])
}

async fn create_price(Json(payload): Json<PriceCreateRequest>) -> Json<Price> {
    Json(Price {
        id: Uuid::new_v4(),
        base_currency: payload.base_currency,
        currency: payload.currency,
        value: payload.value,
        source: payload.source,
        created_at: "2022-01-01T00:00:00Z".to_string(),
        updated_at: "2022-01-01T00:00:00Z".to_string(),
    })
}

async fn get_price_by_id(Path(id): Path<Uuid>) -> Json<Price> {
    // Placeholder: fetch price by ID from your datastore
    Json(Price {
        id,
        base_currency: "USD".to_string(),
        currency: "EUR".to_string(),
        value: 1.1,
        source: "Example Source".to_string(),
        created_at: "2022-01-01T00:00:00Z".to_string(),
        updated_at: "2022-01-02T00:00:00Z".to_string(),
    })
}

async fn update_price(Path(id): Path<Uuid>, Json(payload): Json<PriceUpdateRequest>) -> Json<Price> {
    // Placeholder: update price by ID in your datastore
    Json(Price {
        id,
        base_currency: "USD".to_string(),
        currency: "EUR".to_string(),
        value: payload.value,
        source: payload.source,
        created_at: "2022-01-01T00:00:00Z".to_string(),
        updated_at: "2022-01-02T00:00:00Z".to_string(),
    })
}

async fn delete_price(Path(id): Path<Uuid>) -> StatusCode {
    // Placeholder: delete price by ID from your datastore
    StatusCode::NO_CONTENT
}

async fn get_price_history(Path(currency_pair): Path<String>) -> Json<Vec<PriceHistory>> {
    // Placeholder: fetch price history for a currency pair
    Json(vec![PriceHistory {
        date: "2022-01-01T00:00:00Z".to_string(),
        value: 1.1,
    }])
}

async fn list_price_sources(Path(currency_pair): Path<String>) -> Json<Vec<PriceSource>> {
    // Placeholder: list price sources for a currency pair
    Json(vec![PriceSource {
        name: "Example Source".to_string(),
        reliability: 0.99,
    }])
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Fetch host and port from environment variables, providing default values if not set
    let host = env::var("PRICES_REST_API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("PRICES_REST_API_PORT").unwrap_or_else(|_| "5200".into());

    // Combine host and port into a `SocketAddr`
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().expect("Invalid address");

    // Set up tracing subscriber and axum Router as previously demonstrated

    let app = Router::new()
        .route("/api/prices", get(list_prices).post(create_price))
        .route("/api/prices/:id", get(get_price_by_id).put(update_price).delete(delete_price))
        .route("/api/prices/history/:currencyPair", get(get_price_history))
        .route("/api/prices/sources/:currencyPair", get(list_price_sources))
        .layer(TraceLayer::new_for_http());

    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
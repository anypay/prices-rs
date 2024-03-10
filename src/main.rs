mod websockets;

use std::{env, net::SocketAddr, sync::Arc};

use lapin::{
    Channel, Connection, ConnectionProperties
};
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{StreamExt, SinkExt};
use dotenv::dotenv;
use crate::websockets::session::WebsocketClientSession; // Adjust path according to your module structure

use ::prices::sources; // Import the sources module

mod prisma;
use prisma::PrismaClient;
use prisma_client_rust::{chrono, NewClientError};
use prisma::{prices};


#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::init();
    log::info!("app.starting");

    // Fetch host and port from environment variables, providing default values if not set
    let websockets_host = env::var("PRICES_WEBSOCKETS_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let websockets_port = env::var("PRICES_WEBSOCKETS_PORT").unwrap_or_else(|_| "5210".into());

    // Combine host and port into a `SocketAddr`
    let addr = format!("{}:{}", websockets_host, websockets_port).parse::<SocketAddr>().expect("Invalid address");

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    log::info!("Listening on: {}", addr);
    
    let amqp_url = env::var("AMQP_URL").unwrap_or_else(|_| "amqp://localhost:5672/%2f".into());

    let rabbit_connection = Connection::connect(
        amqp_url.as_str(),
        ConnectionProperties::default(),
    )
    .await
    .expect("Failed to connect to RabbitMQ");

    let client: Result<PrismaClient, NewClientError> = PrismaClient::_builder().build().await;

    match sources::coinmarketcap::get_price("BTC".to_string()).await {

        Ok(price) => {
            log::info!("price.updated currency=BTC base_currency=USD value={:?}", price);            

            let all_prices = client.unwrap().prices().find_many(vec![
                prices::currency::equals("BTC".to_string()),
                prices::base_currency::equals("USD".to_string()),
                prices::source::equals("coinmarketcap.com".to_string())
            ]).exec().await.unwrap();            

            if all_prices.len() == 0 {
 
                log::info!("price.not_found");

                let client: Result<PrismaClient, NewClientError> = PrismaClient::_builder().build().await;

                let new_price = client.unwrap().prices().create(
                    "BTC".to_string(),
                    price,
                    chrono::Utc::now().into(),
                    chrono::Utc::now().into(),
                    "coinmarketcap.com".to_string(),
                    vec![
                        prices::base_currency::set("USD".to_string()),
                    ]
                ).exec().await.unwrap();

                log::info!("price.created currency={} base_currency={} value={:?}", new_price.currency, new_price.base_currency, new_price.value);
            }

            for p in all_prices {
                log::info!("price.found currency={} base_currency={} value={:?}", p.currency, p.base_currency, p.value);
            }

        },
        Err(e) => println!("Error: {}", e),
    }

    while let Ok((stream, _)) = listener.accept().await {
        let channel = rabbit_connection.create_channel().await.expect("Failed to create channel");
        tokio::spawn(handle_connection(stream, channel));
    }


}

async fn handle_connection(stream: tokio::net::TcpStream, channel: Channel) {

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Error during WebSocket handshake: {:?}", e);
            return;
        },
    };

    WebsocketClientSession::new(ws_stream, channel).await;

}

fn convert_string_to_f64(input: &str) -> Result<f64, &'static str> {
    match input.parse::<f64>() {
        Ok(value) => Ok(value),
        Err(_) => Err("Failed to parse string to f64"),
    }
}
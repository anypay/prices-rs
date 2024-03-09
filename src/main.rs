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
use prices::sources::coinmarketcap; // Import the sources module

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

    match coinmarketcap::get_price("BTC".to_string()).await {
        Ok(price) => {
            log::info!("price.updated currency=BTC base_currency=USD value={:?}", price);
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

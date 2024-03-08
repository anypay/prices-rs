
use futures_util::{SinkExt, StreamExt};
use lapin::{
    options::{QueueDeclareOptions, QueueDeleteOptions},
    Channel, ConnectionProperties,
};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use uuid::Uuid;
use std::sync::Arc;
use futures_util::{stream::SplitStream, stream::SplitSink};
use tokio_tungstenite::WebSocketStream;

pub struct WebsocketClientSession {
    uid: String,
    ws_read: Arc<Mutex<SplitStream<WebSocketStream<tokio::net::TcpStream>>>>,
    ws_write: Arc<Mutex<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>>>,
    channel: Channel,
}

impl WebsocketClientSession {
    pub async fn new(ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, channel: Channel) -> Self {
        let uid = Uuid::new_v4().to_string();

        // Example: setting up the AMQP queue based on the session UID
        let _queue = channel
            .queue_declare(
                &format!("websocket:{}", uid),
                QueueDeclareOptions {
                    durable: false, // Do not survive broker restart
                    auto_delete: true, // Automatically delete when no consumers
                    exclusive: true, // Exclusive to this connection
                    ..QueueDeclareOptions::default()
                },
                lapin::types::FieldTable::default(),
            )
            .await
            .expect("Queue creation failed");

        let (write, read) = ws.split();
        let ws_read = Arc::new(Mutex::new(read));
        let ws_write = Arc::new(Mutex::new(write));

        let session = WebsocketClientSession {
            uid,
            ws_read,
            ws_write,    
            channel,
        };

        session.on_open().await;

        session
    }

    pub async fn on_open(&self) {
        log::info!("Session opened: {}", self.uid);

        let queue_name = format!("websocket:{}", self.uid);
        let consumer = self.channel
            .basic_consume(
                &queue_name,
                "my_consumer", // Consumer tag, unique string to identify the consumer
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await
            .expect("Failed to create consumer");

        let ws_clone = self.ws_write.clone();
        tokio::spawn(async move {
            let mut consumer = consumer;
            while let Some(delivery) = consumer.next().await {
                match delivery {
                    Ok((delivery)) => {
                        let msg = delivery.data.clone(); // Clone the data for further use
                        if let Ok(text) = String::from_utf8(msg) {
                            let mut ws_guard = ws_clone.lock().await;
                            let send_result = ws_guard.send(Message::Text(text)).await;
                            if let Err(e) = send_result {
                                log::error!("Failed to send message to WebSocket client: {:?}", e);
                                break; // Exit the loop if sending fails
                            }
                        }
            
                        // Acknowledge the message processing
                        let ack_result = delivery.ack(lapin::options::BasicAckOptions::default()).await;
                        if let Err(e) = ack_result {
                            log::error!("Failed to ack message: {:?}", e);
                        }
                    },
                    Err(e) => log::error!("Consumer error: {:?}", e),
                }
            }
        });

            // Here, you can enter a loop to read messages from the WebSocket and handle them
        while let Some(msg) = self.ws_read.lock().await.next().await {
            match msg {
                Ok(message) => self.on_message(message).await,
                Err(e) => {
                    log::error!("Error in WebSocket connection: {:?}", e);
                    break;
                },
            }
        }

        // Make sure to handle session closure outside of the loop
        self.close_session().await;
    }

    pub async fn on_message(&self, message: Message) {
        if let Ok(text) = message.to_text() {
            log::info!("Received message: {}", text);
            // Handle message
            // For example, parsing JSON and dispatching based on topic
        }
    }

    pub async fn close_session(&self) {
        // Example: Delete the queue when the session is closed
        let _ = self
            .channel
            .queue_delete(
                &format!("websocket:{}", self.uid),
                QueueDeleteOptions {
                    if_unused: false,
                    if_empty: false,
                    nowait: false,
                },
            )
            .await;
        
        // Attempt to close the WebSocket connection by sending a close frame
        let mut ws_guard = self.ws_write.lock().await;

            _ = ws_guard.send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Session closed".into(),
            }))).await;
        
        log::info!("Session closed: {}", self.uid);
    }

    // Function to send messages over the WebSocket connection
    async fn send_message(&self, topic: &str, payload: &str) {
        let message = serde_json::json!({
            "topic": topic,
            "payload": payload
        })
        .to_string();

        self.ws_write.lock().await.send(Message::Text(message)).await.expect("Failed to send message");
    }

    // Additional methods to handle AMQP messages, bind queues, etc., can be added here
}

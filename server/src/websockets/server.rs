use crate::dtos::messages::NewMessageRequest;
use crate::environment::Env;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

type WebSocketClientList = Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>;

pub async fn launch_websocket_server(database_pool: PgPool) {
    let server_address = Env::websocket_url();
    let tcp_listener = TcpListener::bind(server_address.clone())
        .await
        .expect("Failed to bind WebSocket server");
    let active_clients = Arc::new(Mutex::new(Vec::new()));

    println!("WebSocket server running on ws://{}", server_address);

    while let Ok((tcp_stream, _)) = tcp_listener.accept().await {
        let shared_clients = active_clients.clone();
        let cloned_db_pool = database_pool.clone();

        tokio::spawn(async move {
            if let Ok(websocket_stream) = accept_async(tcp_stream).await {
                process_client_connection(websocket_stream, shared_clients, cloned_db_pool).await;
            }
        });
    }
}

async fn process_client_connection(
    websocket_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    active_clients: WebSocketClientList,
    database_pool: PgPool,
) {
    let (mut websocket_writer, mut websocket_reader) = websocket_stream.split();

    let (outgoing_sender, mut outgoing_receiver) = mpsc::unbounded_channel();
    active_clients.lock().unwrap().push(outgoing_sender);

    tokio::spawn(async move {
        while let Some(Ok(incoming_message)) = websocket_reader.next().await {
            if incoming_message.is_text() {
                let text_content = incoming_message.to_text().unwrap();
                println!("Received message: {}", text_content);

                if let Ok(parsed_request) = serde_json::from_str::<NewMessageRequest>(text_content)
                {
                    println!("Parsed message: {:?}", parsed_request);

                    if parsed_request.message_type == "file" {
                        if let Some(ref file_content) = parsed_request.file_data {
                            if let Some(ref file_name) = parsed_request.file_path {
                                if let Err(save_error) =
                                    save_file_to_server(file_content, file_name).await
                                {
                                    eprintln!("Failed to save file: {}", save_error);
                                }
                            } else {
                                eprintln!("File path not provided");
                            }
                        }
                    }

                    if let Err(db_error) =
                        save_message_to_database(&database_pool, &parsed_request).await
                    {
                        eprintln!("Failed to save message: {}", db_error);
                    }

                    let response_data = serde_json::json!(parsed_request);
                    let response_text = response_data.to_string();

                    for client_sender in active_clients.lock().unwrap().iter() {
                        let _ = client_sender.send(Message::Text(response_text.clone()));
                    }
                } else {
                    eprintln!("Failed to parse message");
                }
            }
        }
    });

    while let Some(outgoing_message) = outgoing_receiver.recv().await {
        let _ = websocket_writer.send(outgoing_message).await;
    }
}

async fn save_message_to_database(
    database_pool: &PgPool,
    message_request: &NewMessageRequest,
) -> Result<(), sqlx::Error> {
    let db_query_result = sqlx::query!(
        r#"
        INSERT INTO messages (chat_id, user_id, content, created_at, file_path, message_type)
        VALUES ($1, $2, $3, NOW(), $4, $5)
        "#,
        message_request.chat_id,
        message_request.user_id,
        message_request.content,
        message_request.file_path.as_deref(),
        message_request.message_type
    )
    .execute(database_pool)
    .await;

    match db_query_result {
        Ok(_) => {
            println!("Message saved successfully");
            Ok(())
        }
        Err(error) => {
            eprintln!("Failed to save message: {}", error);
            Err(error)
        }
    }
}

async fn save_file_to_server(
    encoded_file_data: &str,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let complete_file_path = format!("uploads/{}", file_name);

    if let Some(parent_directory) = Path::new(&complete_file_path).parent() {
        if !parent_directory.exists() {
            fs::create_dir_all(parent_directory)?;
        }
    }

    let base64_content = encoded_file_data
        .split(',')
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid file data"))?;

    let decoded_file_data = base64::prelude::BASE64_STANDARD
        .decode(base64_content)
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to decode base64")
        })?;

    let mut file_writer = File::create(&complete_file_path)?;
    file_writer.write_all(&decoded_file_data)?;

    println!("File saved: {}", complete_file_path);

    Ok(())
}

mod protocol;

use clap::{Arg, ArgAction, Command as ClapCommand};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

use concurrent_pqueue::PQueue;
use protocol::*;

/// Main entry point for the PQueue TCP server.
/// Binds to the specified host/port and accepts client connections,
/// spawning a new async task for each client connection.
#[tokio::main]
async fn main() {
    let matches = ClapCommand::new("PQueue Server")
        .version("0.1.0")
        .author("Your Name")
        .about("Asynchronous priority queue server")
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Sets the host address")
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the port to bind")
                .default_value("8002"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Output extra debugging info to stdout")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<String>("port").unwrap();
    let debug = matches.get_flag("debug");
    let address = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&address).await.unwrap();
    println!("Server running on {}", address);

    let pqueue = Arc::new(PQueue::<String>::new()); // Replace String with your item type

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pqueue_clone = pqueue.clone();

        tokio::spawn(async move {
            handle_connection(socket, pqueue_clone, debug).await;
        });
    }
}

/// Handles a single client connection with line-based protocol parsing.
/// Reads commands terminated by CRLF and responds with the appropriate results.
/// Each client gets a unique UUID for debug logging.
async fn handle_connection(mut socket: TcpStream, pqueue: Arc<PQueue<String>>, debug: bool) {
    let client_id = Uuid::new_v4();
    if debug {
        println!("[{}] client connected", client_id)
    }
    let mut buffer = Vec::new();
    let mut char_buffer = [0; 1];

    loop {
        // Read one byte at a time to detect CRLF line endings
        match socket.read_exact(&mut char_buffer).await {
            Ok(_) => {
                // Protocol uses CRLF (\r\n) line termination
                if char_buffer == [b'\n'] && buffer.last() == Some(&b'\r') {
                    // Remove the last character (CR)
                    buffer.pop();

                    // Convert buffer to string
                    let command_string = String::from_utf8_lossy(&buffer);

                    if debug {
                        println!("[{}] rcv: {}", client_id, &command_string);
                    }
                    // Process the command
                    let command = Command::from(command_string.as_ref());
                    let result = process_command(command, &pqueue);

                    let resp = result.to_string();

                    if debug {
                        println!("[{}]snd: {}", client_id, &resp);
                    }

                    // Send response
                    if let Err(e) = socket.write_all(resp.as_bytes()).await {
                        println!("[{}] Failed to write to socket: {}", client_id, e);
                        return;
                    }

                    // Clear buffer for next command
                    buffer.clear();
                } else {
                    // Not CRLF, keep collecting characters
                    buffer.push(char_buffer[0]);
                }
            }
            Err(_) => {
                if debug {
                    println!("[{}] client disconnected", client_id);
                }
                return;
            }
        }
    }
}

/// Processes a parsed command against the priority queue and returns the appropriate response.
/// Uses -1 as a sentinel value for "not found" or "empty" responses.
fn process_command(command: Command, pqueue: &Arc<PQueue<String>>) -> Response {
    match command {
        Command::Update { item_id, value } => {
            pqueue.update(item_id.into(), value);
            Response::Ok
        }
        Command::Next => pqueue
            .next()
            // Return -1 sentinel for empty queue
            .map_or(Response::Item("-1".to_string()), |item| {
                Response::Item(item)
            }),
        Command::Peek => pqueue
            .peek()
            // Return -1 sentinel for empty queue
            .map_or(Response::Item("-1".to_string()), |item| {
                Response::Item(item)
            }),
        Command::Score { item_id } => pqueue
            .score(&item_id)
            // Return -1 sentinel for item not found
            .map_or(Response::Score(-1), Response::Score),
        Command::Info => Response::Stats(pqueue.stats()),
        Command::Error { msg } => Response::Error(msg),
        Command::Help => Response::Help,
    }
}

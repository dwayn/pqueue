mod protocol;

use clap::{Arg, Command as ClapCommand};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt as _, AsyncReadExt as _}};
use std::sync::Arc;

use protocol::*;
use pqueue::PQueue;


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
        .get_matches();

        let host = matches.get_one::<String>("host").unwrap();
        let port = matches.get_one::<String>("port").unwrap();
        let address = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&address).await.unwrap();
    println!("Server running on {}", address);

    let pqueue = Arc::new(PQueue::<String>::new()); // Replace String with your item type

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pqueue_clone = pqueue.clone();

        tokio::spawn(async move {
            handle_connection(socket, pqueue_clone).await;
        });
    }
}


async fn handle_connection(mut socket: TcpStream, pqueue: Arc<PQueue<String>>) {
    let mut buffer = Vec::new();
    let mut char_buffer = [0; 1];

    loop {
        // Read one byte (character) at a time
        match socket.read_exact(&mut char_buffer).await {
            Ok(_) => {
                // Check for CRLF
                if char_buffer == [b'\n'] && buffer.last() == Some(&b'\r') {
                    // Remove the last character (CR)
                    buffer.pop();

                    // Convert buffer to string
                    let command_string = String::from_utf8_lossy(&buffer);

                    println!("rcv: {}", &command_string);
                    // Process the command
                    let command = Command::from(command_string.as_ref());
                    let result = process_command(command, &pqueue);

                    let resp = result.to_string();
                    println!("snd: {}", &resp);
                    // Send response
                    if let Err(e) = socket.write_all(resp.as_bytes()).await {
                        println!("Failed to write to socket: {}", e);
                        return;
                    }

                    // Clear buffer for next command
                    buffer.clear();
                } else {
                    // Not CRLF, keep collecting characters
                    buffer.push(char_buffer[0]);
                }
            }
            Err(e) => {
                println!("Failed to read from socket: {}", e);
                return;
            }
        }
    }
}

fn process_command(command: Command, pqueue: &Arc<PQueue<String>>) -> Response {
    match command {
        Command::Update { item_id, value } => {
            pqueue.update(item_id.into(), value);
            Response::Ok
        },
        Command::Next => {
            pqueue.next().map_or(Response::Item("-1".to_string()), |item| Response::Item(item))
        },
        Command::Peek => {
            pqueue.peek().map_or(Response::Item("-1".to_string()), |item| Response::Item(item))
        },
        Command::Score { item_id } => {
            pqueue.score(&item_id).map_or(Response::Score(-1), Response::Score)
        },
        Command::Info => {
            Response::Stats(pqueue.stats()) // Assuming stats() method returns PQueueStats
        },
        Command::Error { msg } => {
            Response::Error(msg)
        },
    }
}

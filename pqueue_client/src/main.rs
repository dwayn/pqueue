use clap::{Arg, Command as ClapCommand, value_parser};
use tokio::{net::TcpStream, io::{AsyncWriteExt as _, AsyncReadExt as _}};

#[tokio::main]
async fn main() {
    let matches = ClapCommand::new("PQueue Client")
        .version("1.0")
        .author("Your Name")
        .about("Client for PQueue Server")
        .arg(Arg::new("host")
            .long("host")
            .default_value("localhost")
            .help("Server host address"))
        .arg(Arg::new("port")
            .long("port")
            .default_value("8002")
            .help("Server port"))
        .subcommand(ClapCommand::new("update")
            .about("Update an item's score")
            .arg(Arg::new("item_id").required(true))
            .arg(Arg::new("value").value_parser(value_parser!(i64)).required(true)))
        .subcommand(ClapCommand::new("next").about("Get the next item"))
        .subcommand(ClapCommand::new("peek").about("Peek at the next item"))
        .subcommand(ClapCommand::new("score")
            .about("Get an item's score")
            .arg(Arg::new("item_id").required(true)))
        .subcommand(ClapCommand::new("info").about("Get server info"))
        .get_matches();

    let server_address = format!(
        "{}:{}",
        matches.get_one::<String>("host").unwrap(),
        matches.get_one::<String>("port").unwrap()
    );

    match matches.subcommand() {
        Some(("update", sub_m)) => {
            let item_id = sub_m.get_one::<String>("item_id").unwrap();
            let value = sub_m.get_one::<i64>("value").unwrap();
            let command = format!("UPDATE {} {}", item_id, value);
            send_command(&server_address, &command).await.unwrap();
        },
        Some(("next", _)) => {
            send_command(&server_address, "NEXT").await.unwrap();
        },
        Some(("peek", _)) => {
            send_command(&server_address, "PEEK").await.unwrap();
        },
        Some(("score", sub_m)) => {
            let item_id = sub_m.get_one::<String>("item_id").unwrap();
            let command = format!("SCORE {}", item_id);
            send_command(&server_address, &command).await.unwrap();
        },
        Some(("info", _)) => {
            send_command(&server_address, "INFO").await.unwrap();
        },
        _ => eprintln!("Invalid command"),
    }
}

use tokio::time::{timeout, Duration};

async fn send_command(server_address: &str, command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(server_address).await?;
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;

    let mut response = String::new();

    // Set a timeout for the read operation
    let duration = Duration::from_millis(1000);  // Set to 5 seconds, adjust as needed
    match timeout(duration, stream.read_to_string(&mut response)).await {
        Ok(_) => Ok(response),
        Err(_) => Ok(response),
    }
}

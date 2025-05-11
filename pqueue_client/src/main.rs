use clap::{Arg, ArgAction, Command};
use tokio::{
    io::{self, AsyncBufReadExt as _, AsyncWriteExt},
    net::TcpStream,
    select,
};

#[tokio::main]
async fn main() {
    let matches = Command::new("PQueue Interactive Client")
        .arg(Arg::new("host").long("host").default_value("localhost"))
        .arg(Arg::new("port").long("port").default_value("8002"))
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
    let server_address = format!("{}:{}", host, port);

    let mut stream = TcpStream::connect(server_address).await.unwrap();
    // let (mut reader, mut writer) = stream.split();

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let is_interactive = atty::is(atty::Stream::Stdin);

    let (reader, writer) = stream.split();
    let mut reader = io::BufReader::new(reader).lines();
    let mut writer = io::BufWriter::new(writer);
    let mut stdout = io::stdout();

    loop {
        if is_interactive {
            print!("pqueue::{}:{}> ", host, port);
            io::stdout().flush().await.unwrap(); // Ensure the prompt is displayed immediately
        }

        select! {
            command = stdin.next_line() => {
                let command = command.unwrap();

                if let Some(command) = command {
                    let command = command.trim();
                    if !command.is_empty() {
                        if debug { println!("read command: {}", command); }

                        writer.write_all(command.as_bytes()).await.unwrap();
                        writer.write_all(b"\r\n").await.unwrap();
                        writer.flush().await.unwrap();
                    }
                } else {
                    // if user sends ctrl + d or an EOF is streamed in over stdin, the stdin reader will have
                    // a None value and we can break out
                    return;
                }
            }
            response = reader.next_line() => {
                let response = response.unwrap();
                if let Some(response) = response {
                    if debug { println!("received response: {}", response); }

                    stdout.write_all(&response.as_bytes()).await.unwrap();
                    stdout.write_all(b"\n").await.unwrap();
                    stdout.flush().await.unwrap();
                } else {
                    // If we get an EOF or the socket is disconnected, flow ends up here and we can break out
                    return;
                }

            }
        }
    }
}

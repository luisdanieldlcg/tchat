pub mod args;
use args::{ClientArgs, TChatArgs};
use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, TcpListener, TcpStream};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::args::Mode;

#[tokio::main]
async fn main() {
    let args = TChatArgs::parse();
    TChat::start(args).await;
}

struct Config {
    username: String,
    socket: Socket,
}

struct TChat;

impl TChat {
    pub async fn start(args: TChatArgs) {
        let addr = match &args.mode {
            Mode::Connect(_) => "127.0.0.1:0".to_string(),
            Mode::Serve(args) => format!("{}:{}", args.addr, args.port),
        };

        let config = Config {
            username: args.username,
            socket: Self::bind(&addr),
        };

        println!("Running TChat CLI App in {}", args.mode.as_str());
        println!("Connected as {}", config.username);

        match args.mode {
            Mode::Connect(args) => Self::run_client(config, args).await,
            Mode::Serve(_) => Self::run_server(config).await,
        };
    }

    pub fn bind(ipv4_addr: &str) -> Socket {
        let addr = ipv4_addr
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| panic!("{} is not a valid IPV4 address", ipv4_addr));

        let socket = match Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)) {
            Ok(t) => t,
            Err(e) => panic!("Couldn't bind socket: {}", e),
        };
        if let Err(e) = socket.bind(&addr.into()) {
            panic!("Failed to bind socket: {} at {}", e, addr);
        }
        socket
    }

    pub async fn run_client(config: Config, args: ClientArgs) {
        println!(
            "TChat socket bound to: {}",
            config.socket.local_addr().unwrap().as_socket().unwrap()
        );
        let tgt_addr = format!("{}:{}", args.addr, args.port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| panic!("{} is not a valid IPV4 address", args.addr));

        config
            .socket
            .connect(&tgt_addr.into())
            .expect("Failed to connect to server");

        println!("Connection established at {}", tgt_addr);

        Self::handle_connect_tcp(config).await;
    }

    pub async fn run_server(config: Config) {
        config.socket.listen(128).unwrap();
        let addr = config.socket.local_addr().unwrap().as_socket().unwrap();
        println!("Server listening at: {}", addr);
        Self::handle_serve_tcp(config).await;
    }

    pub async fn handle_serve_tcp(config: Config) {
        let tcp: TcpListener = config.socket.into();

        for req in tcp.incoming() {
            match req {
                Ok(stream) => {
                    let (mut reader, mut writer) = tokio::net::TcpStream::from_std(stream)
                        .unwrap()
                        .into_split();

                    let read_handle = tokio::spawn(async move {
                        tokio::io::copy(&mut reader, &mut tokio::io::stdout())
                            .await
                            .unwrap();
                    });
                    let username = config.username.clone();
                    let write_handle = tokio::spawn(async move {
                        Self::send_message(&username, &mut writer).await;
                    });

                    tokio::select! {
                        _ = read_handle => {

                        },
                        _ = write_handle => {

                        }
                    };
                }
                Err(e) => eprintln!("Failed to receive request: {}", e),
            }
        }
    }

    pub async fn handle_connect_tcp(config: Config) {
        let tcp: TcpStream = config.socket.into();
        let stream = tokio::net::TcpStream::from_std(tcp).expect("Failed to connect to server");

        let (mut reader, mut writer) = stream.into_split();

        let client_read =
            tokio::spawn(
                async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await },
            );

        let username = config.username.clone();
        let client_write = tokio::spawn(async move {
            Self::send_message(&username, &mut writer).await;
        });

        tokio::select! {
            _ = client_read => {

            },
            _ = client_write => {

            }
        };
    }

    pub async fn send_message(username: &str, writer: &mut OwnedWriteHalf) {
        let prefix = format!("[{}]: ", username);
        let input = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(input);
        let mut buffer = String::new();
        while let Ok(n) = reader.read_line(&mut buffer).await {
            if n == 0 {
                break;
            }
            buffer.insert_str(0, &prefix);
            writer.write_all(buffer.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
            buffer.clear();
        }
    }

}

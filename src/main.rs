pub mod args;
use args::{ClientArgs, NetchatArgs, ServerArgs};
use clap::Parser;
use socket2::{Domain, Socket, Type, Protocol};
use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::args::Mode;

struct Netchat {
    username: String,
}

impl Netchat {
    
    pub async fn start(args: NetchatArgs) {
        
        let addr = match &args.mode {
            Mode::Connect(_) => "127.0.0.1:0".to_string(),
            Mode::Serve(args) => format!("{}:{}", args.addr, args.port),
        };
       
        let socket = Self::bind(&addr);
        println!("Running Netchat CLI App in {}", args.mode.as_str());

        let this = Self {
            username: args.username,
        };

        match args.mode {
            Mode::Connect(args) => this.run_client(args, socket).await,
            Mode::Serve(args) => this.run_server(args, socket).await,
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

        if let Err(e) =  socket.bind(&addr.into()) {
            panic!("Failed to bind socket: {} at {}", e, addr);
        }
        socket
    }

    pub async fn run_client(&self, args: ClientArgs, socket: Socket) {
        println!(
            "Netchat socket bound to: {}",
            socket.local_addr().unwrap().as_socket().unwrap()
        );
        let tgt_addr = format!("{}:{}", args.addr, args.port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| panic!("{} is not a valid IPV4 address", args.addr));

        socket
            .connect(&tgt_addr.into())
            .expect("Failed to connect to server");
        println!("Connection established at {}", tgt_addr);

        let tcp: TcpStream = socket.into();
        let stream = tokio::net::TcpStream::from_std(tcp).expect("Failed to connect to server");

        let (mut reader, mut writer) = stream.into_split();

        let client_read =
            tokio::spawn(
                async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await },
            );

        let username = self.username.clone();
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

    pub async fn run_server(&self, args: ServerArgs, socket: Socket) {
        socket.listen(128).unwrap();
        let addr = socket.local_addr().unwrap().as_socket().unwrap();
        println!("Server listening at: {}", addr);
        let tcp: TcpListener = socket.into();

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
                    let username = self.username.clone();
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
}

#[tokio::main]
async fn main() {
    let args = NetchatArgs::parse();
    Netchat::start(args).await;
}

use anyhow::Result;
use bubbles_core::{
    bubble::Bubble,
    web::{Request, Response},
};
use std::{fs::File, net::SocketAddr};
use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};

const BUBBLE_FILE: &str = "/home/jack/Documents/bubbles/test_bubbles.json";

async fn recv_request(socket: &mut TcpStream) -> Result<Request> {
    let mut socket = BufStream::new(socket);

    // let mut line = vec![];
    let mut line = String::new();
    socket.read_line(&mut line).await?;

    // let request = serde_json::from_slice(&line)?;
    let request = serde_json::from_str(&line)?;

    Ok(request)
}

async fn send_response(socket: &mut TcpStream, response: &Response) -> Result<()> {
    let mut buff = BufStream::new(socket);
    let data = serde_json::to_string(response)?;
    println!("Sending data: {data}");
    buff.write_all(data.as_bytes()).await?;
    buff.flush().await?;
    Ok(())
}

async fn process_socket(mut socket: TcpStream) -> Result<()> {
    let request = recv_request(&mut socket).await?;
    println!("Recv request");

    let file = File::open(BUBBLE_FILE)?;
    let mut bubbles: Vec<Bubble> = serde_json::from_reader(file)?;

    println!("Got request: {request:?}");
    match request {
        Request::Set(v) => {
            bubbles
                .iter_mut()
                .zip(v.into_iter())
                .for_each(|(f, s)| f.days.extend(s.days));

            let file = File::open(BUBBLE_FILE)?;
            serde_json::to_writer(file, &bubbles)?;
        }
        Request::GetInfo => {
            bubbles.iter_mut().for_each(|x| x.days.clear());
            send_response(&mut socket, &Response::Bubbles(bubbles)).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "0.0.0.0:45531".parse::<SocketAddr>()?;
    let listen = TcpListener::bind(addr).await?;
    loop {
        let (socket, _) = listen.accept().await?;
        println!("Got connection!");

        if let Err(e) = process_socket(socket).await {
            println!("error: {e}");
        }
    }
}

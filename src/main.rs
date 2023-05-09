use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::http::StatusCode;
use std::convert::Infallible;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::SinkExt;
use futures_util::StreamExt;

static INDEX_HTML: &str = include_str!("index.html");

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(INDEX_HTML.into())
        .unwrap();
    Ok(response)
}

async fn http_server() {
    let make_svc = make_service_fn(|_conn| {
        let svc = service_fn(handle_request);
        async move { Ok::<_, Infallible>(svc) }
    });

    let addr = ([127, 0, 0, 1], 7777).into();
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn accept_connection(stream: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to accept websocket connection");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received a message: {}", text);
                ws_sender.send(Message::Text(text.to_uppercase())).await.unwrap();
            }
            Ok(Message::Close(_)) => {
                break;
            }
            _ => (),
        }
    }
}

async fn ws_server() {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

#[tokio::main]
async fn main() {
    let http = http_server();
    let ws = ws_server();
    tokio::join!(http, ws);
}
use futures::{sink::SinkExt, stream::StreamExt};
use hyper::{Body, Request, Response};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use std::convert::Infallible;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

static INDEX_HTML: &str = include_str!("index.html");

async fn handle_request(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    if hyper_tungstenite::is_upgrade_request(&request) {
        let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

        tokio::spawn(async move {
            if let Err(e) = serve_websocket(websocket).await {
                eprintln!("Error in websocket connection: {}", e);
            }
        });

        Ok(response)
    } else {
        Ok(Response::new(Body::from(INDEX_HTML)))
    }
}

async fn serve_websocket(websocket: HyperWebsocket) -> Result<(), Error> {
    let mut websocket = websocket.await?;

    while let Some(message) = websocket.next().await {
        match message? {
            tungstenite::Message::Text(text) => {
                println!("Received a message: {}", text);
                websocket.send(tungstenite::Message::Text(text.to_uppercase())).await?;
            }
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 7777).into();
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let mut http = hyper::server::conn::Http::new();
    http.http1_only(true);
    http.http1_keep_alive(true);

    loop {
        let (stream, _) = listener.accept().await?;
        let connection = http
            .serve_connection(stream, hyper::service::service_fn(handle_request))
            .with_upgrades();

        tokio::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving HTTP connection: {:?}", err);
            }
        });
    }
}
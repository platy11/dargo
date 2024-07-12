mod message;
mod wsserver;
mod uinput;

use std::io;
use axum::{extract::MatchedPath, response::IntoResponse, routing, Router};
use tokio::net::TcpListener;

const STATIC_FILES: [(&str, &str, &[u8]); 4] = [
    ("/index.html", "text/html", include_bytes!("../../dargo-client/index.html")),
    ("/index.css", "text/css", include_bytes!("../../dargo-client/index.css")),
    ("/dist/bundle.js", "application/javascript", include_bytes!("../../dargo-client/dist/bundle.js")),
    ("/dist/bundle.js.map", "application/javascript", include_bytes!("../../dargo-client/dist/bundle.js.map"))
];

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut app = Router::new()
        .route("/api/socket", routing::get(wsserver::ws_handler))
        .route("/", routing::get(static_file));
    for (file, _, _) in STATIC_FILES {
        app = app.route(file, routing::get(static_file))
    }

    let bind_addr = std::env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await
}

async fn static_file(mp: MatchedPath) -> axum::response::Response {
    let mut path = mp.as_str();
    if path == "/" {
        path = "/index.html"
    }
    for (file, mime, bytes) in STATIC_FILES {
        if file == path {
            return ([("content-type", mime)], bytes).into_response()
        }
    }
    unreachable!()
}

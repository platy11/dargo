use axum::{extract::ws, response::Response};

use crate::message::DimensionsData;
use crate::message::Message::DimensionsUpdate;
use crate::uinput::UinputTrackpad;

pub async fn ws_handler(ws: ws::WebSocketUpgrade) -> Response {
    ws.on_upgrade(process_socket)
}

async fn process_socket(mut socket: ws::WebSocket) {
    // TODO: we should really have a better logging/error handling setup
    println!("socket established");

    let mut trackpad: Option<UinputTrackpad> = None;
    while let Some(msg) = socket.recv().await {
        let Ok(ws::Message::Text(msg_str)) = msg else {
            println!("received non-text websocket message, ignoring");
            continue
        };
        let Ok(msg_data) = serde_json::from_str(&msg_str) else {
            println!("received invalid websocket message, ignoring");
            continue
        };

        if let Some(ref mut trackpad) = trackpad {
            if let Err(err) = trackpad.process_message(msg_data) {
                println!("failed to process message, ignoring: {}", err)
            }
        } else if let DimensionsUpdate(DimensionsData {width, height, resolution}) = msg_data {
            let tp = UinputTrackpad::new(width, height, resolution);
            if let Err(err) = tp {
                println!("couldn't create trackpad, closing socket: {}", err);
                return
            }
            trackpad = tp.ok()
        } else {
            println!("message received before DimensionsUpdate, ignoring");
        }
    }
}

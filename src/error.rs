use websockets::WebSocketError;
use wamp_helpers::error::Error as WampParseError;

#[derive(Debug)]
pub enum Error {
    NoWebsocketError,
    WsError(WebSocketError),
    JsonError(WampParseError)
}
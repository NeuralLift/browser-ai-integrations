use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;

pub fn sse_event(data: &str) -> Result<Event, Infallible> {
    Ok(Event::default().data(data))
}

pub fn sse_done() -> Result<Event, Infallible> {
    Ok(Event::default().data("[DONE]"))
}

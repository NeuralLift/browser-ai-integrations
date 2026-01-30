use axum::response::sse::Event;
use std::convert::Infallible;

#[allow(dead_code)]
pub fn sse_event(data: &str) -> Result<Event, Infallible> {
    Ok(Event::default().data(data))
}

#[allow(dead_code)]
pub fn sse_done() -> Result<Event, Infallible> {
    Ok(Event::default().data("[DONE]"))
}

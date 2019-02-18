use chrono::prelude::{DateTime, Local};
use warp::http::{header::CONTENT_TYPE, Response, Result};

pub fn local_time_now_to_string() -> String {
    local_time_to_string(Local::now())
}

pub fn local_time_to_string(dt: DateTime<Local>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S%.9f %z").to_string()
}

pub fn html_string_to_response(html: String) -> Result<Response<String>> {
    warp::http::Response::builder()
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.clone())
}

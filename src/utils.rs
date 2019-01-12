use chrono::prelude::{DateTime, Local};

pub fn local_time_now_to_string() -> String {
    local_time_to_string(Local::now())
}

pub fn local_time_to_string(dt: DateTime<Local>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S%.9f %z").to_string()
}

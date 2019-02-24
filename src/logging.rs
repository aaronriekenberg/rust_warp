use chrono::prelude::Local;

use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc;

fn run_logging_output_thread(receiver: mpsc::Receiver<String>) {
    let mut stdout = ::std::io::stdout();

    loop {
        match receiver.recv() {
            Ok(first_msg) => {
                stdout
                    .write(first_msg.as_bytes())
                    .expect("run_logging_output_thread error writing first_msg to stdout");
                loop {
                    match receiver.try_recv() {
                        Ok(next_msg) => {
                            stdout.write(next_msg.as_bytes()).expect(
                                "run_logging_output_thread error writing next_msg to stdout",
                            );
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            stdout
                                .write(
                                    "run_logging_output_thread try_recv disconnected error"
                                        .as_bytes(),
                                )
                                .expect("run_logging_output_thread error writing to stdout");
                        }
                    }
                }
            }
            Err(e) => {
                stdout
                    .write(format!("run_logging_output_thread recv error {}\n", e).as_bytes())
                    .expect("run_logging_output_thread error writing to stdout");
            }
        }

        stdout
            .flush()
            .expect("run_logging_output_thread error flushing stdout");
    }
}

fn get_log_level() -> Result<::log::LevelFilter, Box<::std::error::Error>> {
    let log_level_string = ::std::env::var("LOGGING_LEVEL").unwrap_or("INFO".to_string());
    let log_level = ::log::LevelFilter::from_str(&log_level_string)?;
    Ok(log_level)
}

pub fn initialize_logging() -> Result<(), Box<::std::error::Error>> {
    let (sender, receiver) = mpsc::channel();

    ::std::thread::Builder::new()
        .name("logging_output".to_string())
        .spawn(move || {
            run_logging_output_thread(receiver);
        })?;

    let log_level = get_log_level()?;

    ::fern::Dispatch::new()
        .level(log_level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.9f %z"),
                ::std::thread::current().name().unwrap_or("UNKNOWN"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(sender)
        .apply()?;

    Ok(())
}

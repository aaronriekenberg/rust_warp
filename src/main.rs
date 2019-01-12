mod config;
mod handlers;
mod logging;
mod utils;

use handlebars::Handlebars;

use serde::Serialize;
use serde_json::json;

use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, Filter, Reply};

fn install_panic_hook() {
    let original_panic_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        original_panic_hook(panic_info);
        std::process::exit(1);
    }));
}

fn create_handlebars() -> Arc<Handlebars> {
    let mut mut_hb = Handlebars::new();
    mut_hb.set_strict_mode(true);

    mut_hb
        .register_template_file("index.html", "./templates/index.hbs")
        .expect("failed to register index.html");

    mut_hb
        .register_template_file("command.html", "./templates/command.hbs")
        .expect("failed to register command.html");

    Arc::new(mut_hb)
}

fn static_file_routes() -> BoxedFilter<(impl Reply,)> {
    warp::path("static").and(warp::fs::dir("./static")).boxed()
}

fn main() {
    install_panic_hook();

    logging::initialize_logging().expect("failed to initialize logging");

    let config_file = std::env::args()
        .nth(1)
        .expect("config file required as command line argument");

    let config = config::read_config(config_file).expect("error reading configuration file");

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = create_handlebars();

    let routes =
        handlers::index::create_routes(Arc::clone(&hb), config.main_page_info(), config.commands())
            .or(handlers::command::create_routes(
                Arc::clone(&hb),
                config.commands(),
            ))
            .or(static_file_routes())
            .with(warp::log("main"));

    let listen_addr: std::net::SocketAddr = config
        .server_info()
        .listen_address()
        .parse()
        .expect("listen_addr parse error");

    warp::serve(routes)
        .tls("localhost-cert.pem", "localhost-privkey.pem")
        .run(listen_addr);
}

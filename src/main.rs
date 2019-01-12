mod config;
mod handlers;
mod logging;
mod utils;

use handlebars::Handlebars;

use serde::Serialize;
use serde_json::json;

use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, path, Filter, Reply};

fn install_panic_hook() {
    let original_panic_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        original_panic_hook(panic_info);
        std::process::exit(1);
    }));
}

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars>) -> impl warp::Reply
where
    T: Serialize,
{
    hbs.render(template.name, &template.value)
        .unwrap_or_else(|err| err.description().to_owned())
}

fn create_handlebars() -> Arc<Handlebars> {
    let mut mut_hb = Handlebars::new();

    mut_hb
        .register_template_file("command.html", "./templates/command.hbs")
        .expect("failed to register command.html");
    mut_hb
        .register_template_file("index.html", "./templates/index.hbs")
        .expect("failed to register index.html");

    Arc::new(mut_hb)
}

fn index_route(hb: Arc<Handlebars>) -> BoxedFilter<(impl Reply,)> {
    // Create a reusable closure to render template
    let handlebars_handler = move |with_template| render(with_template, Arc::clone(&hb));

    //GET /
    warp::get2()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: "index.html",
            value: json!({"user" : "Warp"}),
        })
        .map(handlebars_handler)
        .boxed()
}

fn favicon_route() -> BoxedFilter<(impl Reply,)> {
    warp::get2()
        .and(warp::path("favicon.ico"))
        .and(warp::fs::file("./static/rust-favicon.ico"))
        .boxed()
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

    let routes = index_route(Arc::clone(&hb))
        .or(handlers::command::html_route(
            Arc::clone(&hb),
            config.commands(),
        ))
        .or(handlers::command::api_route(config.commands()))
        .or(favicon_route())
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

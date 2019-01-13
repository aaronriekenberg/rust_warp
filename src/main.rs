mod config;
mod handlers;
mod logging;
mod utils;

use handlebars::Handlebars;

use std::sync::Arc;

use warp::Filter;

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

    handlers::index::register_templates(&mut mut_hb).expect("error registering index templates");
    handlers::command::register_templates(&mut mut_hb)
        .expect("error registering command templates");
    handlers::proxy::register_templates(&mut mut_hb).expect("error registering proxy templates");

    Arc::new(mut_hb)
}

fn main() {
    install_panic_hook();

    logging::initialize_logging().expect("failed to initialize logging");

    let config_file = std::env::args()
        .nth(1)
        .expect("config file required as command line argument");

    let config = config::read_config(config_file).expect("error reading configuration file");

    let hb = create_handlebars();

    let routes = handlers::index::create_routes(
        Arc::clone(&hb),
        config.main_page_info(),
        config.commands(),
        config.proxies(),
    )
    .or(handlers::command::create_routes(
        Arc::clone(&hb),
        config.commands(),
    ))
    .or(handlers::proxy::create_routes(
        Arc::clone(&hb),
        config.proxies(),
    ))
    .or(handlers::static_file::create_routes())
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

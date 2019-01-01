mod logging;

use warp::Filter;

fn main() {
    logging::initialize_logging().expect("failed to initialize logging");

    // Match any request and return hello world!
    let routes = warp::any().map(|| "Hello, World!").with(warp::log("main"));

    warp::serve(routes)
        .tls("localhost-cert.pem", "localhost-privkey.pem")
        .run(([127, 0, 0, 1], 3030));
}

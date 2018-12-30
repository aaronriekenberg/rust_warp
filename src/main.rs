use warp::Filter;

fn main() {
    pretty_env_logger::init();

    // Match any request and return hello world!
    let routes = warp::any().map(|| "Hello, World!").with(warp::log("main"));

    warp::serve(routes)
        .tls("localhost-cert.pem", "localhost-privkey.pem")
        .run(([127, 0, 0, 1], 3030));
}

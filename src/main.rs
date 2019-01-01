mod logging;

use log::warn;

use handlebars::Handlebars;

use serde::Serialize;
use serde_json::json;

use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, path, Filter, Future, Reply};

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
    // register the template
    mut_hb
        .register_template_file("index.html", "./templates/index.hbs")
        .expect("failed to register template file");

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

fn api_route() -> BoxedFilter<(impl Reply,)> {
    warp::get2()
        .and(path!("api" / String))
        .and_then(|user| {
            futures::future::ok::<String, String>(format!("hello {}", user)).map_err(|err| {
                warn!("future error {}", err);
                warp::reject::custom(err)
            })
        })
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

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = create_handlebars();

    let routes = index_route(Arc::clone(&hb))
        .or(api_route())
        .or(favicon_route())
        .with(warp::log("main"));

    warp::serve(routes)
        .tls("localhost-cert.pem", "localhost-privkey.pem")
        .run(([127, 0, 0, 1], 3030));
}

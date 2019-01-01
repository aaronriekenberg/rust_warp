mod logging;

use handlebars::Handlebars;

use serde::Serialize;
use serde_json::json;

use std::error::Error;
use std::sync::Arc;

use warp::Filter;

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

fn main() {
    install_panic_hook();

    logging::initialize_logging().expect("failed to initialize logging");

    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_file("index.html", "./templates/index.hbs")
        .expect("failed to register template file");

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);

    // Create a reusable closure to render template
    let handlebars = move |with_template| render(with_template, hb.clone());

    //GET /
    let route = warp::get2()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: "index.html",
            value: json!({"user" : "Warp"}),
        })
        .map(handlebars)
        .with(warp::log("main"));

    warp::serve(route)
        .tls("localhost-cert.pem", "localhost-privkey.pem")
        .run(([127, 0, 0, 1], 3030));
}

use handlebars::Handlebars;

use serde_derive::Serialize;

use std::error::Error;
use std::process::Command;
use std::sync::Arc;

use tokio_process::CommandExt;

use warp::{filters::BoxedFilter, path, Filter, Future, Reply};

fn build_html_response(
    hb: Arc<Handlebars>,
    main_page_info: &crate::config::MainPageInfo,
) -> String {
    hb.render("index.html", &main_page_info)
        .unwrap_or_else(|err| err.description().to_owned())
}

pub fn create_routes(
    hb: Arc<Handlebars>,
    main_page_info: &crate::config::MainPageInfo,
) -> BoxedFilter<(impl Reply,)> {
    let html = build_html_response(hb, &main_page_info);

    warp::get2()
        .and(warp::path::end())
        .map(move || html.clone())
        .boxed()
}

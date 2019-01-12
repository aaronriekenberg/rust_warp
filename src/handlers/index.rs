use handlebars::Handlebars;

use serde_derive::Serialize;

use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, Filter, Future, Reply};

#[derive(Serialize)]
struct TemplateData {
    main_page_info: crate::config::MainPageInfo,
    commands: Vec<crate::config::CommandInfo>,
}

fn build_html_response(hb: Arc<Handlebars>, template_data: TemplateData) -> String {
    hb.render("index.html", &template_data)
        .unwrap_or_else(|err| err.description().to_owned())
}

pub fn create_routes(
    hb: Arc<Handlebars>,
    main_page_info: &crate::config::MainPageInfo,
    commands: &Vec<crate::config::CommandInfo>,
) -> BoxedFilter<(impl Reply,)> {
    let template_data = TemplateData {
        main_page_info: main_page_info.clone(),
        commands: commands.clone(),
    };

    let html = build_html_response(hb, template_data);

    warp::get2()
        .and(warp::path::end())
        .map(move || html.clone())
        .boxed()
}

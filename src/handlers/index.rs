use handlebars::Handlebars;

use serde_derive::Serialize;

use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, Filter, Reply};

static INDEX_TEMPLATE_NAME: &'static str = "index.html";
static INDEX_TEMPLATE_PATH: &'static str = "./templates/index.hbs";

#[derive(Serialize)]
struct TemplateData {
    main_page_info: crate::config::MainPageInfo,
    commands: Vec<crate::config::CommandInfo>,
    proxies: Vec<crate::config::ProxyInfo>,
}

pub fn register_templates(mut_hb: &mut Handlebars) -> Result<(), handlebars::TemplateFileError> {
    mut_hb.register_template_file(INDEX_TEMPLATE_NAME, INDEX_TEMPLATE_PATH)
}

fn build_html_response(hb: Arc<Handlebars>, template_data: TemplateData) -> String {
    hb.render(INDEX_TEMPLATE_NAME, &template_data)
        .unwrap_or_else(|err| err.description().to_owned())
}

pub fn create_routes(
    hb: Arc<Handlebars>,
    main_page_info: &crate::config::MainPageInfo,
    commands: &Vec<crate::config::CommandInfo>,
    proxies: &Vec<crate::config::ProxyInfo>,
) -> BoxedFilter<(impl Reply,)> {
    let template_data = TemplateData {
        main_page_info: main_page_info.clone(),
        commands: commands.clone(),
        proxies: proxies.clone(),
    };

    let html = build_html_response(hb, template_data);

    warp::get2()
        .and(warp::path::end())
        .map(move || html.clone())
        .boxed()
}

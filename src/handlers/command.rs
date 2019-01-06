use handlebars::Handlebars;

use serde::Serialize;
use serde_json::json;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, path, Filter, Rejection, Reply};

type CommandMap = HashMap<String, crate::config::CommandInfo>;

fn build_command_map(commands: &Vec<crate::config::CommandInfo>) -> CommandMap {
    let mut map = CommandMap::new();
    for command_info in commands {
        map.insert(command_info.id().clone(), command_info.clone());
    }
    map
}

fn build_response(
    command_id: String,
    command_map: &CommandMap,
    hb: Arc<Handlebars>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match command_map.get(&command_id) {
        None => Err(warp::reject::not_found()),
        Some(command_info) => Ok(hb
            .render("command.html", &command_info)
            .unwrap_or_else(|err| err.description().to_owned())),
    }
}

pub fn html_route(
    hb: Arc<Handlebars>,
    commands: &Vec<crate::config::CommandInfo>,
) -> BoxedFilter<(impl Reply,)> {
    let command_map = build_command_map(commands);

    warp::get2()
        .and(path!("commands" / String))
        .and_then(move |command_id| build_response(command_id, &command_map, Arc::clone(&hb)))
        .boxed()
}

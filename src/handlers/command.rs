//use futures::Future;

use handlebars::Handlebars;

use serde_derive::Serialize;
use serde_json::json;

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::sync::Arc;

use tokio_process::CommandExt;

use warp::{filters::BoxedFilter, path, Filter, Future, Rejection, Reply};

type CommandMap = HashMap<String, crate::config::CommandInfo>;

fn build_command_map(commands: &Vec<crate::config::CommandInfo>) -> CommandMap {
    let mut map = CommandMap::new();
    for command_info in commands {
        map.insert(command_info.id().clone(), command_info.clone());
    }
    map
}

fn build_html_response(
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
        .and_then(move |command_id| build_html_response(command_id, &command_map, Arc::clone(&hb)))
        .boxed()
}

fn run_command(
    command_info: crate::config::CommandInfo,
) -> Box<Future<Item = String, Error = std::io::Error> + Send> {
    let mut command = Command::new(command_info.command());

    command.args(command_info.args());

    command
        .output_async()
        .and_then(move |output| {
            let mut combined_output =
                String::with_capacity(output.stderr.len() + output.stdout.len());
            combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
            combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
            Ok(combined_output)
        })
        .or_else(move |err| Ok(format!("command error: {}", err)))
        .boxed()
}

fn build_command_line_string(command_info: &crate::config::CommandInfo) -> String {
    let mut command_line_string = String::new();

    command_line_string.push_str(command_info.command());

    for arg in command_info.args() {
        command_line_string.push(' ');
        command_line_string.push_str(arg);
    }

    command_line_string
}

#[derive(Serialize)]
struct APIResponse {
    now: String,
    command_line: String,
    output: String,
}

pub fn api_route(commands: &Vec<crate::config::CommandInfo>) -> BoxedFilter<(impl Reply,)> {
    let command_map = build_command_map(commands);

    warp::get2()
        .and(path!("api" / "commands" / String))
        .and_then(move |command_id| match command_map.get(&command_id) {
            None => futures::future::err(warp::reject::not_found()).boxed(),
            // Some(command_info) => {
            //     let api_response = APIResponse {
            //             now: "now".to_string(),
            //             command_line: "cl".to_string(),
            //             output: "output".to_string()
            //     };
            //     Box::new(futures::future::ok(warp::reply::json(&api_response)))
            // }
            Some(command_info) => {
                let command_info_clone = command_info.clone();
                let command_line_string = build_command_line_string(&command_info_clone);

                run_command(command_info_clone)
                    .and_then(move |command_output| {
                        let api_response = APIResponse {
                            now: crate::utils::local_time_now_to_string(),
                            command_line: command_line_string,
                            output: command_output,
                        };
                        Ok(warp::reply::json(&api_response))
                    })
                    .or_else(|err| Err(warp::reject::server_error()))
                    .boxed()
            }
        })
        .boxed()
}

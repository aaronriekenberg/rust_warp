use handlebars::Handlebars;

use serde_derive::Serialize;

use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::sync::Arc;

use tokio_process::CommandExt;

use warp::{filters::BoxedFilter, path, Filter, Future, Reply};

type HTMLMap = HashMap<String, String>;

fn build_html_map(hb: Arc<Handlebars>, commands: &Vec<crate::config::CommandInfo>) -> HTMLMap {
    let mut map = HTMLMap::new();
    for command_info in commands {
        map.insert(
            command_info.id().clone(),
            hb.render("command.html", &command_info)
                .unwrap_or_else(|err| err.description().to_owned()),
        );
    }
    map
}

fn html_route(
    hb: Arc<Handlebars>,
    commands: &Vec<crate::config::CommandInfo>,
) -> BoxedFilter<(impl Reply,)> {
    let html_map = build_html_map(hb, commands);

    warp::get2()
        .and(path!("commands" / String))
        .and_then(move |command_id| match html_map.get(&command_id) {
            None => Err(warp::reject::not_found()),
            Some(html) => Ok(html.clone()),
        })
        .boxed()
}

type CommandMap = HashMap<String, crate::config::CommandInfo>;

fn build_command_map(commands: &Vec<crate::config::CommandInfo>) -> CommandMap {
    let mut map = CommandMap::new();
    for command_info in commands {
        map.insert(command_info.id().clone(), command_info.clone());
    }
    map
}

fn run_command(
    command_info: crate::config::CommandInfo,
) -> Box<Future<Item = String, Error = std::io::Error> + Send> {
    let mut command = Command::new(command_info.command());

    command.args(command_info.args());

    Box::new(
        command
            .output_async()
            .and_then(move |output| {
                let mut combined_output =
                    String::with_capacity(output.stderr.len() + output.stdout.len());
                combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                Ok(combined_output)
            })
            .or_else(move |err| Ok(format!("command error: {}", err))),
    )
}

#[derive(Serialize)]
struct APIResponse {
    now: String,
    command: String,
    args: Vec<String>,
    output: String,
}

fn build_command_api_response(
    command_info_option: Option<&crate::config::CommandInfo>,
) -> Box<Future<Item = impl warp::Reply, Error = warp::Rejection> + Send> {
    match command_info_option {
        None => Box::new(futures::future::err(warp::reject::not_found())),
        Some(command_info) => {
            let command_clone = command_info.command().clone();
            let args_clone = command_info.args().clone();

            Box::new(
                run_command(command_info.clone())
                    .and_then(move |command_output| {
                        let api_response = APIResponse {
                            now: crate::utils::local_time_now_to_string(),
                            command: command_clone,
                            args: args_clone,
                            output: command_output,
                        };
                        Ok(warp::reply::json(&api_response))
                    })
                    .or_else(|err| Err(warp::reject::custom(err))),
            )
        }
    }
}

fn api_route(commands: &Vec<crate::config::CommandInfo>) -> BoxedFilter<(impl Reply,)> {
    let command_map = build_command_map(commands);

    warp::get2()
        .and(path!("api" / "commands" / String))
        .and_then(move |command_id| build_command_api_response(command_map.get(&command_id)))
        .boxed()
}

pub fn create_routes(
    hb: Arc<Handlebars>,
    commands: &Vec<crate::config::CommandInfo>,
) -> BoxedFilter<(impl Reply,)> {
    html_route(hb, commands).or(api_route(commands)).boxed()
}

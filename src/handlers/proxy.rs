use handlebars::Handlebars;

use hyper::rt::Stream;
use hyper::Uri;

use serde_derive::Serialize;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use warp::{filters::BoxedFilter, path, Filter, Future, Reply};

static PROXY_TEMPLATE_NAME: &'static str = "proxy.html";
static PROXY_TEMPLATE_PATH: &'static str = "./templates/proxy.hbs";

pub fn register_templates(mut_hb: &mut Handlebars) -> Result<(), handlebars::TemplateFileError> {
    mut_hb.register_template_file(PROXY_TEMPLATE_NAME, PROXY_TEMPLATE_PATH)
}

type HTMLMap = HashMap<String, String>;

fn build_html_map(hb: Arc<Handlebars>, proxies: &Vec<crate::config::ProxyInfo>) -> HTMLMap {
    let mut map = HTMLMap::new();
    for proxy_info in proxies {
        map.insert(
            proxy_info.id().clone(),
            hb.render(PROXY_TEMPLATE_NAME, &proxy_info)
                .unwrap_or_else(|err| err.description().to_owned()),
        );
    }
    map
}

fn html_route(
    hb: Arc<Handlebars>,
    proxies: &Vec<crate::config::ProxyInfo>,
) -> BoxedFilter<(impl Reply,)> {
    let html_map = build_html_map(hb, proxies);

    warp::get2()
        .and(path!("proxies" / String))
        .and_then(move |proxy_id| match html_map.get(&proxy_id) {
            None => Err(warp::reject::not_found()),
            Some(html) => Ok(html.clone()),
        })
        .boxed()
}

type HyperHttpClient = ::hyper::Client<
    ::hyper::client::HttpConnector<::hyper::client::connect::dns::TokioThreadpoolGaiResolver>,
    ::hyper::Body,
>;

#[derive(Clone)]
struct ProxyInfoAndURI {
    proxy_info: crate::config::ProxyInfo,
    uri: Uri,
}

type ProxyMap = HashMap<String, ProxyInfoAndURI>;

fn build_proxy_map(proxies: &Vec<crate::config::ProxyInfo>) -> ProxyMap {
    let mut map = ProxyMap::new();
    for proxy_info in proxies {
        map.insert(
            proxy_info.id().clone(),
            ProxyInfoAndURI {
                proxy_info: proxy_info.clone(),
                uri: proxy_info.url().parse().expect("error parsing proxy url"),
            },
        );
    }
    map
}

#[derive(Default)]
struct ResponseInfo {
    version: String,
    status: String,
    headers: String,
    body: String,
}

fn fetch_proxy(
    uri: &Uri,
    http_client: &HyperHttpClient,
) -> Box<Future<Item = ResponseInfo, Error = std::io::Error> + Send> {
    Box::new(
        http_client
            .get(uri.clone())
            .and_then(move |response| {
                let version = format!("{:?}", response.version());
                let status = format!("{}", response.status());
                let headers = format!("{:#?}", response.headers());
                response
                    .into_body()
                    .concat2()
                    .then(move |result| match result {
                        Ok(body) => Ok(ResponseInfo {
                            version,
                            status,
                            headers,
                            body: String::from_utf8_lossy(&body).into_owned(),
                        }),
                        Err(e) => Ok(ResponseInfo {
                            version,
                            status,
                            headers,
                            body: format!("proxy body error: {}", e),
                        }),
                    })
            })
            .or_else(move |err| {
                Ok(ResponseInfo {
                    body: format!("proxy error: {}", err),
                    ..Default::default()
                })
            }),
    )
}

#[derive(Serialize)]
struct APIResponse {
    now: String,
    method: String,
    url: String,
    version: String,
    status: String,
    headers: String,
    body: String,
}

fn build_proxy_api_response(
    http_client: &HyperHttpClient,
    proxy_info_and_uri_option: Option<&ProxyInfoAndURI>,
) -> Box<Future<Item = impl warp::Reply, Error = warp::Rejection> + Send> {
    match proxy_info_and_uri_option {
        None => Box::new(futures::future::err(warp::reject::not_found())),
        Some(proxy_info_and_uri) => {
            let uri_clone = proxy_info_and_uri.uri.clone();

            Box::new(
                fetch_proxy(&proxy_info_and_uri.uri, &http_client)
                    .and_then(move |response_info| {
                        let api_response = APIResponse {
                            now: crate::utils::local_time_now_to_string(),
                            method: "GET".to_string(),
                            url: uri_clone.to_string(),
                            version: response_info.version,
                            status: response_info.status,
                            headers: response_info.headers,
                            body: response_info.body,
                        };

                        Ok(warp::reply::json(&api_response))
                    })
                    .or_else(|err| Err(warp::reject::custom(err))),
            )
        }
    }
}

fn api_route(proxies: &Vec<crate::config::ProxyInfo>) -> BoxedFilter<(impl Reply,)> {
    let proxy_map = build_proxy_map(proxies);
    let http_connector = ::hyper::client::HttpConnector::new_with_tokio_threadpool_resolver();
    let http_client = ::hyper::client::Client::builder().build(http_connector);

    warp::get2()
        .and(path!("api" / "proxies" / String))
        .and_then(move |proxy_id| build_proxy_api_response(&http_client, proxy_map.get(&proxy_id)))
        .boxed()
}

pub fn create_routes(
    hb: Arc<Handlebars>,
    proxies: &Vec<crate::config::ProxyInfo>,
) -> BoxedFilter<(impl Reply,)> {
    html_route(hb, proxies).or(api_route(proxies)).boxed()
}

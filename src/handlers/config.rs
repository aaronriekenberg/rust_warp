use warp::{filters::BoxedFilter, Filter, Reply};

pub fn create_routes(config: &crate::config::Configuration) -> BoxedFilter<(impl Reply,)> {
    let config_string = format!("{:#?}", config);

    warp::get2()
        .and(warp::path("configuration"))
        .map(move || config_string.clone())
        .boxed()
}

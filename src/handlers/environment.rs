use warp::{filters::BoxedFilter, Filter, Reply};

pub fn create_routes(environment: &crate::environment::Environment) -> BoxedFilter<(impl Reply,)> {
    let environment_string = format!("{:#?}", environment);

    warp::get2()
        .and(warp::path("environment"))
        .map(move || environment_string.clone())
        .boxed()
}

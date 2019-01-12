use warp::{filters::BoxedFilter, Filter, Future, Reply};

pub fn create_routes() -> BoxedFilter<(impl Reply,)> {
    let static_route = warp::path("static").and(warp::fs::dir("./static"));

    let favicon_route = warp::path("favicon.ico").and(warp::fs::file("./static/favicon.ico"));

    static_route.or(favicon_route).boxed()
}

mod extract;
mod middleware;
mod reqwest;
mod router;

use ::reqwest::Client;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::CONFIG;

pub fn routes() -> Router<Arc<Client>> {
    let mut get_gh = get(router::get_gh);
    if let Some(token) = CONFIG.token.clone() {
        debug!("TokenLayer");
        get_gh = get_gh.route_layer(ValidateRequestHeaderLayer::custom(middleware::Token::new(
            token,
        )));
    }
    Router::new().route("/*gh_path", get(get_gh))
}

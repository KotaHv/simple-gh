mod extract;
mod middleware;
mod reqwest;
mod router;

use ::reqwest::Client;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::CONFIG;
use middleware::TokenLayer;

pub fn routes() -> Router<Arc<Client>> {
    let mut get_gh = get(router::get_gh);
    if CONFIG.token.is_some() {
        debug!("TokenLayer");
        get_gh = get_gh.route_layer(TokenLayer);
    }
    Router::new().route("/*gh_path", get(get_gh))
}

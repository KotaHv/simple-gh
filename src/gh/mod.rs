use rocket::Route;

mod guard;
mod reqwest;
mod router;

pub fn routes() -> Vec<Route> {
    routes![router::get_gh]
}

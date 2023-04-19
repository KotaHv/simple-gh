use actix_web::{get, http::StatusCode, web, HttpResponse, Responder};
use reqwest::Client;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gh);
}

#[derive(Debug)]
struct RequestError {
    reason: String,
    status: StatusCode,
}

struct Request {
    url: String,
    client: web::Data<Client>,
}

impl Request {
    fn new(client: web::Data<Client>, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    async fn get(&self) -> Result<reqwest::Response, RequestError> {
        match self.client.get(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(RequestError {
                    reason: e.to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }

    async fn head(&self) -> Result<reqwest::Response, RequestError> {
        match self.client.head(&self.url).send().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{}: {:?}", self.url, e);
                return Err(RequestError {
                    reason: e.to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }
}

#[get("/{gh_path:.*}")]
async fn get_gh(gh_path: web::Path<String>, client: web::Data<Client>) -> impl Responder {
    let req = Request::new(client, &gh_path);
    let res = match req.get().await {
        Ok(res) => res,
        Err(re) => return HttpResponse::build(re.status).body(re.reason),
    };
    let status_code = res.status();
    let content_type = match res.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap(),
        None => "application/octet-stream",
    }
    .to_string();
    let content = res.bytes().await.unwrap();
    HttpResponse::build(status_code)
        .append_header(("content-type", content_type))
        .body(content)
}

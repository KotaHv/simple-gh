use std::str::FromStr;

use rocket::{http::ContentType, Request};

pub fn content_type(res: &reqwest::Response) -> ContentType {
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    match ContentType::from_str(content_type) {
        Ok(ct) => ct,
        Err(e) => {
            error!("{} content-type: {e}", res.url());
            ContentType::Bytes
        }
    }
}

pub fn get_ip(request: &Request) -> String {
    let mut ip = "Unknown".to_string();
    let ip_option = request.client_ip();
    if ip_option.is_some() {
        ip = ip_option.unwrap().to_string();
    }
    ip
}

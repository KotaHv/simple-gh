use actix_web::{http::header, HttpRequest};

pub fn get_ip(req: &HttpRequest) -> String {
    let connection_info = req.connection_info();
    if let Some(ip) = connection_info.realip_remote_addr() {
        return ip.to_string();
    }
    if let Some(ip) = connection_info.peer_addr() {
        return ip.to_string();
    }
    "-".to_string()
}

pub fn get_header(req: &HttpRequest, key: impl header::AsHeaderName) -> String {
    if let Some(header) = req.headers().get(key) {
        if let Ok(header) = header.to_str() {
            return header.to_string();
        }
    }
    "-".to_string()
}

pub fn get_ua(req: &HttpRequest) -> String {
    get_header(req, header::USER_AGENT)
}

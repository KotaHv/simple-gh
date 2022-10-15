pub fn content_type(res: &reqwest::Response) -> Result<(String, String), String> {
    let content_type_string = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let content_type_option = content_type_string.split_once("/");
    if let Some(content_type) = content_type_option {
        Ok((content_type.0.to_string(), content_type.1.to_string()))
    } else {
        Err(content_type_string)
    }
}

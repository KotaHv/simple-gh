use std::sync::Arc;

use reqwest::Client;

use crate::CustomError;

type RequestOutput = Result<reqwest::Response, CustomError>;

pub struct Request {
    url: String,
    client: Arc<Client>,
}
impl Request {
    pub fn new(client: Arc<Client>, gh_path: &str) -> Self {
        Request {
            url: format!("https://raw.githubusercontent.com/{gh_path}"),
            client,
        }
    }
    pub async fn get(&self) -> RequestOutput {
        Request::result(self.client.get(&self.url).send().await)
    }

    pub async fn head(&self) -> RequestOutput {
        Request::result(self.client.head(&self.url).send().await)
    }

    fn result(res: Result<reqwest::Response, reqwest::Error>) -> RequestOutput {
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{:#?}", e);
                Err(CustomError::reason(e.to_string()))
            }
        }
    }
}

use super::client_trait::ClientTrait;
use reqwest;

mod constants {
    use lazy_static::lazy_static;
    use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};

    lazy_static! {
        pub static ref HTTP_HEADER: HeaderMap = {
            let mut headers = HeaderMap::with_capacity(2);
            headers.insert(ACCEPT, "application/json".parse().unwrap());
            headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
            headers
        };
    }
}

pub struct Client {
    pub login: String,
    pub password: String,
}

impl Client {
    pub fn new(login: &str, password: &str) -> Self {
        Self {
            login: login.to_string(),
            password: password.to_string(),
        }
    }
}

impl ClientTrait for Client {
    fn request_get<Url: reqwest::IntoUrl + 'static>(
        &self,
        url: Url,
    ) -> reqwest::Result<reqwest::blocking::Response> {
        reqwest::blocking::Client::new()
            .get(url)
            .headers(constants::HTTP_HEADER.clone())
            .basic_auth(&self.login, Some(&self.password))
            .send()
    }
}

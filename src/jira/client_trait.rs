use reqwest::{blocking::Response, IntoUrl, Result};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait ClientTrait {
    fn request_get<Url: IntoUrl + 'static>(&self, url: Url) -> Result<Response>;
}

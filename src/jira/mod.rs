mod client;
mod client_trait;
mod user_worklogs;
mod user_worklogs_fetcher;
mod user_worklogs_fetcher_trait;
mod worklog;

pub use client::Client;
pub use user_worklogs::UserWorklogs;
pub use user_worklogs_fetcher::UserWorklogsFetcher;
pub use user_worklogs_fetcher_trait::UserWorklogsFetcherTrait;
pub use worklog::Worklog;

#[cfg(test)]
pub mod testing {
    pub use super::client_trait::MockClientTrait;
    pub use super::user_worklogs_fetcher_trait::MockUserWorklogsFetcherTrait;
}

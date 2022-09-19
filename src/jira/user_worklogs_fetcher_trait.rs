use super::user_worklogs::UserWorklogs;
use anyhow::Result;
use chrono::NaiveDate;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait UserWorklogsFetcherTrait {
    fn fetch(
        &self,
        user_name: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<UserWorklogs>;
}

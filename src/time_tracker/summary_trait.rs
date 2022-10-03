use super::user_worklogs_summary::UserWorklogsSummary;
#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait SummaryTrait {
    fn get_user_worklogs_summary(
        &self,
        user_name: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> anyhow::Result<UserWorklogsSummary>;
}

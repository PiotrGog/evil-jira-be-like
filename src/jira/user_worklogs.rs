use super::worklog::Worklog;
use chrono::NaiveDate;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserWorklogs {
    pub user: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub worklogs: Vec<Worklog>,
}

impl UserWorklogs {
    pub fn new(
        user: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        worklogs: Vec<Worklog>,
    ) -> Self {
        Self {
            user: user.to_string(),
            start_date: start_date,
            end_date: end_date,
            worklogs: worklogs,
        }
    }
}

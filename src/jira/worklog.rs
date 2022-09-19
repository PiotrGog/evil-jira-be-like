use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Worklog {
    pub data: chrono::NaiveDate,
    pub issue_key: String,
    pub issue_summary: String,
    pub time_spent: chrono::Duration,
}

impl Worklog {
    pub fn new(
        data: chrono::NaiveDate,
        issue_key: &str,
        issue_summary: &str,
        time_spent: chrono::Duration,
    ) -> Self {
        Self {
            data: data,
            issue_key: issue_key.to_string(),
            issue_summary: issue_summary.to_string(),
            time_spent: time_spent,
        }
    }

    // fn time_in_hours(&self) -> u64
}

impl Display for Worklog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seconds = self.time_spent.num_seconds() % 60;
        let minutes = self.time_spent.num_minutes() % 60;
        let hours = self.time_spent.num_hours();
        write!(
            f,
            "Issue key: {}, Summary: {}, Logger time: {}h{}m{}s",
            self.issue_key, self.issue_summary, hours, minutes, seconds
        )
    }
}

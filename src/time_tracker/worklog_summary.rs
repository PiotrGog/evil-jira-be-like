use crate::jira::Worklog;
use chrono::Duration;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WorklogSummary {
    pub spent_time: Duration,
    pub worklogs: Vec<Worklog>,
}

use super::worklog_summary::WorklogSummary;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserWorklogsSummary(pub HashMap<NaiveDate, WorklogSummary>);

impl Deref for UserWorklogsSummary {
    type Target = HashMap<NaiveDate, WorklogSummary>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserWorklogsSummary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl UserWorklogsSummary {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

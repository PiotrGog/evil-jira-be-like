use crate::jira::UserWorklogsFetcherTrait;
use chrono::Duration;
use itertools::Itertools;

use super::{user_worklogs_summary::UserWorklogsSummary, worklog_summary::WorklogSummary};

pub struct Summary<UserWorklogsFetcherType>
where
    UserWorklogsFetcherType: UserWorklogsFetcherTrait,
{
    user_worklog_fetcher: UserWorklogsFetcherType,
}

impl<UserWorklogsFetcherType> Summary<UserWorklogsFetcherType>
where
    UserWorklogsFetcherType: UserWorklogsFetcherTrait,
{
    pub fn new(user_worklog_fetcher: UserWorklogsFetcherType) -> Self {
        Self {
            user_worklog_fetcher,
        }
    }

    pub fn get_user_worklogs_summary(
        &self,
        user_name: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> anyhow::Result<UserWorklogsSummary> {
        let user_worklogs = self
            .user_worklog_fetcher
            .fetch(user_name, start_date, end_date)?;

        let user_worklogs_by_date = user_worklogs
            .worklogs
            .into_iter()
            .into_group_map_by(|worklog| worklog.data);

        let summary = start_date
            .iter_days()
            .take_while(|date| date.le(&end_date))
            .map(|date| {
                let user_worklog_in_the_date = user_worklogs_by_date
                    .get(&date)
                    .cloned()
                    .unwrap_or_default();
                (
                    date,
                    WorklogSummary {
                        spent_time: user_worklog_in_the_date
                            .iter()
                            .fold(Duration::seconds(0), |acc, worklog| {
                                acc + worklog.time_spent
                            }),
                        worklogs: user_worklog_in_the_date.to_owned(),
                    },
                )
            })
            .collect();

        Ok(UserWorklogsSummary(summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::{
        user_worklogs_fetcher_trait::MockUserWorklogsFetcherTrait, UserWorklogs, Worklog,
    };
    use anyhow::anyhow;
    use chrono::{Duration, NaiveDate};
    use mockall::predicate::eq;

    #[test]
    fn get_user_worklogs_summary_returns_error_if_worklog_fetcher_fail() {
        let user_name = "dummy_user_name";
        let start_date = NaiveDate::from_ymd(2022, 09, 10);
        let end_date = NaiveDate::from_ymd(2022, 09, 19);

        let error_message = "some dummy error message";

        let mut user_worklog_fetcher = MockUserWorklogsFetcherTrait::new();
        user_worklog_fetcher
            .expect_fetch()
            .with(eq(user_name), eq(start_date), eq(end_date))
            .return_once(move |_, _, _| Err(anyhow!(error_message)));

        let uut = Summary::new(user_worklog_fetcher);

        let result = uut.get_user_worklogs_summary(user_name, start_date, end_date);
        assert!(result.is_err());
    }

    #[test]
    fn get_user_worklogs_summary_return_valid_summary() {
        let user_name = "dummy_user_name";
        let start_date = NaiveDate::from_ymd(2022, 09, 10);
        let end_date = NaiveDate::from_ymd(2022, 09, 19);
        let other_date = NaiveDate::from_ymd(2022, 09, 15);

        let dummy_duration1 = Duration::minutes(20);
        let dummy_duration2 = Duration::hours(4);
        let dummy_duration3 = Duration::seconds(20);
        let dummy_duration4 = Duration::hours(1);

        let worklog1_start_date = Worklog::new(
            start_date,
            "start_date-key1",
            "start_date-summary1",
            dummy_duration1,
        );
        let worklog1_end_date = Worklog::new(
            end_date,
            "end_date-key1",
            "end_date-summary1",
            dummy_duration2,
        );
        let worklog2_end_date = Worklog::new(
            end_date,
            "end_date-key2",
            "end_date-summary2",
            dummy_duration3,
        );
        let worklog1_other_date = Worklog::new(
            other_date,
            "other_date-key1",
            "other_date-summary1",
            dummy_duration1,
        );
        let worklog2_other_date = Worklog::new(
            other_date,
            "other_date-key2",
            "other_date-summary2",
            dummy_duration2,
        );
        let worklog3_other_date = Worklog::new(
            other_date,
            "other_date-key3",
            "other_date-summary3",
            dummy_duration3,
        );
        let worklog4_other_date = Worklog::new(
            other_date,
            "other_date-key4",
            "other_date-summary4",
            dummy_duration4,
        );

        let user_worklogs = UserWorklogs::new(
            user_name,
            start_date,
            end_date,
            vec![
                worklog1_start_date.clone(),
                worklog1_end_date.clone(),
                worklog2_end_date.clone(),
                worklog1_other_date.clone(),
                worklog2_other_date.clone(),
                worklog3_other_date.clone(),
                worklog4_other_date.clone(),
            ],
        );

        let mut user_worklog_fetcher = MockUserWorklogsFetcherTrait::new();
        user_worklog_fetcher
            .expect_fetch()
            .with(eq(user_name), eq(start_date), eq(end_date))
            .return_once(move |_, _, _| Ok(user_worklogs));

        let uut = Summary::new(user_worklog_fetcher);

        let mut expected_result = UserWorklogsSummary::new();
        start_date
            .iter_days()
            .take_while(|date| date.le(&end_date))
            .for_each(|date| {
                expected_result.insert(
                    date,
                    WorklogSummary {
                        spent_time: Duration::seconds(0),
                        worklogs: vec![],
                    },
                );
            });
        expected_result.insert(
            start_date,
            WorklogSummary {
                spent_time: dummy_duration1,
                worklogs: vec![worklog1_start_date],
            },
        );
        expected_result.insert(
            other_date,
            WorklogSummary {
                spent_time: dummy_duration1 + dummy_duration2 + dummy_duration3 + dummy_duration4,
                worklogs: vec![
                    worklog1_other_date,
                    worklog2_other_date,
                    worklog3_other_date,
                    worklog4_other_date,
                ],
            },
        );
        expected_result.insert(
            end_date,
            WorklogSummary {
                spent_time: dummy_duration2 + dummy_duration3,
                worklogs: vec![worklog1_end_date, worklog2_end_date],
            },
        );

        let result = uut.get_user_worklogs_summary(user_name, start_date, end_date);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result);
    }
}

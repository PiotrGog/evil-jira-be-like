pub mod audio;
pub mod gui;
pub mod jira;
pub mod time_tracker;

pub mod application {
    use super::audio::PlayerTrait;
    use super::gui::WindowTrait;
    use super::time_tracker::SummaryTrait;
    use crate::time_tracker::UserWorklogsSummary;

    pub fn run(
        summary: impl SummaryTrait,
        mut window: impl WindowTrait,
        player: impl PlayerTrait,
        user_name: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> anyhow::Result<()> {
        match summary.get_user_worklogs_summary(user_name, start_date, end_date) {
            Err(error) => process_error(error),
            Ok(result) => process_ok(result, &mut window, &player)?,
        };
        Ok(())
    }

    fn process_error(error: anyhow::Error) {
        eprintln!("ERROR: {}", error);
    }

    fn process_ok(
        result: UserWorklogsSummary,
        window: &mut impl WindowTrait,
        player: &impl PlayerTrait,
    ) -> anyhow::Result<()> {
        let any_worklog_spent_timme_other_than_expected =
            result.iter().any(|(_worklog_date, worklog_summary)| {
                println!("{}", worklog_summary.spent_time);
                worklog_summary.spent_time != chrono::Duration::hours(8)
            });
        println!("{:?}", any_worklog_spent_timme_other_than_expected);
        if any_worklog_spent_timme_other_than_expected {
            window.load_image("assets/image.jpg")?;
            window.show_image()?;
            player.play()?;
            window.hide_image()?;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::audio;
        use crate::gui;
        use crate::jira;
        use crate::time_tracker;
        use anyhow::anyhow;
        use chrono::Duration;
        use chrono::NaiveDate;
        use lazy_static::lazy_static;
        use mockall::predicate::eq;

        const USER_NAME: &'static str = "user_name";

        lazy_static! {
            static ref START_DATE: NaiveDate = NaiveDate::from_ymd(2022, 09, 10);
            static ref END_DATE: NaiveDate = NaiveDate::from_ymd(2022, 09, 10);
            static ref DATE_1: NaiveDate = NaiveDate::from_ymd(2022, 09, 15);
            static ref DATE_2: NaiveDate = NaiveDate::from_ymd(2022, 09, 16);
            static ref DURATION_1H: Duration = Duration::hours(1);
            static ref DURATION_3H: Duration = Duration::hours(3);
            static ref DURATION_4H: Duration = Duration::hours(4);
            static ref DURATION_5H: Duration = Duration::hours(5);
        }

        #[test]
        fn run_function_does_nothing_when_summary_is_empty() {
            let sin_player = audio::testing::MockPlayerTrait::default();
            let windows = gui::testing::MockWindowTrait::default();
            let mut summary = time_tracker::testing::MockSummaryTrait::default();

            summary
                .expect_get_user_worklogs_summary()
                .with(eq(USER_NAME), eq(*START_DATE), eq(&*END_DATE))
                .return_once(move |_, _, _| Ok(UserWorklogsSummary::new()));

            run(
                summary,
                windows,
                sin_player,
                USER_NAME,
                *START_DATE,
                *END_DATE,
            )
            .unwrap();
        }

        #[test]
        fn run_function_does_nothing_when_summary_contains_logged_data_with_matching_expected_time()
        {
            let mut sin_player = audio::testing::MockPlayerTrait::default();
            let mut windows = gui::testing::MockWindowTrait::default();
            let mut summary = time_tracker::testing::MockSummaryTrait::default();

            let date_1_worklog1 =
                jira::Worklog::new(*DATE_1, "date_1-key1", "date_1-summary1", *DURATION_3H);
            let date_1_worklog2 =
                jira::Worklog::new(*DATE_1, "date_1-key1", "date_1-summary1", *DURATION_5H);
            let date_2_worklog1 =
                jira::Worklog::new(*DATE_2, "date_2-key1", "date_2-summary1", *DURATION_1H);
            let date_2_worklog2 =
                jira::Worklog::new(*DATE_2, "date_2-key2", "date_2-summary2", *DURATION_3H);
            let date_2_worklog3 =
                jira::Worklog::new(*DATE_2, "date_2-key3", "date_2-summary3", *DURATION_4H);

            let mut user_worklog_summary = UserWorklogsSummary::new();
            user_worklog_summary.insert(
                *DATE_1,
                time_tracker::WorklogSummary {
                    spent_time: *DURATION_3H + *DURATION_5H,
                    worklogs: vec![date_1_worklog1, date_1_worklog2],
                },
            );
            user_worklog_summary.insert(
                *DATE_2,
                time_tracker::WorklogSummary {
                    spent_time: *DURATION_1H + *DURATION_3H + *DURATION_4H,
                    worklogs: vec![date_2_worklog1, date_2_worklog2, date_2_worklog3],
                },
            );
            summary
                .expect_get_user_worklogs_summary()
                .with(eq(USER_NAME), eq(*START_DATE), eq(*END_DATE))
                .return_once(move |_, _, _| Ok(user_worklog_summary));

            windows.expect_load_image().times(0);
            windows.expect_show_image().times(0);
            sin_player.expect_play().times(0);
            windows.expect_hide_image().times(0);

            run(
                summary,
                windows,
                sin_player,
                USER_NAME,
                *START_DATE,
                *END_DATE,
            )
            .unwrap();
        }

        #[test]
        fn run_function_show_message_window_when_summary_contains_logged_data_with_no_matching_expected_time(
        ) {
            let mut sin_player = audio::testing::MockPlayerTrait::default();
            let mut windows = gui::testing::MockWindowTrait::default();
            let mut summary = time_tracker::testing::MockSummaryTrait::default();

            let date_1_worklog1 =
                jira::Worklog::new(*DATE_1, "date_1-key1", "date_1-summary1", *DURATION_3H);
            let date_1_worklog2 =
                jira::Worklog::new(*DATE_1, "date_1-key1", "date_1-summary1", *DURATION_5H);
            let date_2_worklog2 =
                jira::Worklog::new(*DATE_2, "date_2-key2", "date_2-summary2", *DURATION_3H);
            let date_2_worklog3 =
                jira::Worklog::new(*DATE_2, "date_2-key3", "date_2-summary3", *DURATION_4H);

            let mut user_worklog_summary = UserWorklogsSummary::new();
            user_worklog_summary.insert(
                *DATE_1,
                time_tracker::WorklogSummary {
                    spent_time: *DURATION_3H + *DURATION_5H,
                    worklogs: vec![date_1_worklog1, date_1_worklog2],
                },
            );
            user_worklog_summary.insert(
                *DATE_2,
                time_tracker::WorklogSummary {
                    spent_time: *DURATION_3H + *DURATION_4H,
                    worklogs: vec![date_2_worklog2, date_2_worklog3],
                },
            );

            summary
                .expect_get_user_worklogs_summary()
                .with(eq(USER_NAME), eq(*START_DATE), eq(*END_DATE))
                .return_once(move |_, _, _| Ok(user_worklog_summary));

            windows.expect_load_image().return_once(|_| Ok(()));
            windows.expect_show_image().return_once(|| Ok(()));
            sin_player.expect_play().return_once(|| Ok(()));
            windows.expect_hide_image().return_once(|| Ok(()));

            run(
                summary,
                windows,
                sin_player,
                USER_NAME,
                *START_DATE,
                *END_DATE,
            )
            .unwrap();
        }

        #[test]
        fn run_function_report_error_when_summary_is_err() {
            let sin_player = audio::testing::MockPlayerTrait::default();
            let windows = gui::testing::MockWindowTrait::default();
            let mut summary = time_tracker::testing::MockSummaryTrait::default();

            summary
                .expect_get_user_worklogs_summary()
                .with(eq(USER_NAME), eq(*START_DATE), eq(*END_DATE))
                .return_once(move |_, _, _| Err(anyhow!("Error in get_user_worklogs_summary")));

            run(
                summary,
                windows,
                sin_player,
                USER_NAME,
                *START_DATE,
                *END_DATE,
            )
            .unwrap();
        }
    }
}

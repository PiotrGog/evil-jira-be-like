use super::{
    client_trait::ClientTrait, user_worklogs::UserWorklogs,
    user_worklogs_fetcher_trait::UserWorklogsFetcherTrait,
};
use crate::jira::worklog::Worklog;
use anyhow::anyhow;
use chrono::NaiveDate;
use reqwest::IntoUrl;

const JIRA_TIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3f%z";

pub struct UserWorklogsFetcher<UrlType, ClientType>
where
    UrlType: IntoUrl,
    ClientType: ClientTrait,
{
    jira_api_root_url: UrlType,
    jira_client: ClientType,
}

impl<UrlType, ClientType> UserWorklogsFetcherTrait for UserWorklogsFetcher<UrlType, ClientType>
where
    UrlType: IntoUrl,
    ClientType: ClientTrait,
{
    fn fetch(
        &self,
        user_name: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> anyhow::Result<super::user_worklogs::UserWorklogs> {
        let issues_response = self
            .fetch_issues(user_name, &start_date, &end_date)?
            .error_for_status()?;
        let issues_body = issues_response.json::<serde_json::Value>()?;

        let worklogs = issues_body["issues"]
            .as_array()
            .ok_or(anyhow!("Can't parse 'issues' as array"))?
            .into_iter()
            .map(|issue| self.process_issue(issue, user_name, &start_date, &end_date))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(UserWorklogs::new(user_name, start_date, end_date, worklogs))
    }
}

impl<UrlType, ClientType> UserWorklogsFetcher<UrlType, ClientType>
where
    UrlType: IntoUrl,
    ClientType: ClientTrait,
{
    pub fn new(jira_api_root_url: UrlType, jira_client: ClientType) -> Self {
        Self {
            jira_api_root_url,
            jira_client,
        }
    }

    fn create_jira_issues_endpoint(
        &self,
        user_name: &str,
        start_date: &chrono::NaiveDate,
        end_date: &chrono::NaiveDate,
    ) -> String {
        let date_in_query_format = "%Y-%m-%d";
        format!(
            "/search?jql=worklogDate%20%3E=%20{}%20and%20worklogDate%20%3C=%20{}%20and%20worklogAuthor%20in%20({})",
            start_date.format(date_in_query_format),
            end_date.format(date_in_query_format),
            user_name,
        )
    }

    fn fetch_issues(
        &self,
        user_name: &str,
        start_date: &chrono::NaiveDate,
        end_date: &chrono::NaiveDate,
    ) -> reqwest::Result<reqwest::blocking::Response> {
        let jira_issues_request_url = format!(
            "{}{}",
            self.jira_api_root_url.as_str(),
            self.create_jira_issues_endpoint(user_name, &start_date, &end_date)
        );
        self.jira_client.request_get(jira_issues_request_url)
    }

    fn fetch_issue_worklogs(
        &self,
        issue_url: &str,
    ) -> reqwest::Result<reqwest::blocking::Response> {
        let jira_issue_worklog_url = format!("{}/worklog", issue_url);
        self.jira_client.request_get(jira_issue_worklog_url)
    }

    fn process_issue(
        &self,
        issue: &serde_json::Value,
        user_name: &str,
        start_date: &chrono::NaiveDate,
        end_date: &chrono::NaiveDate,
    ) -> anyhow::Result<Vec<Worklog>> {
        let worklogs_response = self
            .fetch_issue_worklogs(issue["self"].as_str().unwrap())?
            .error_for_status()?;
        let worklogs_response_body = worklogs_response.json::<serde_json::Value>()?;

        let worklogs = worklogs_response_body["worklogs"]
            .as_array()
            .ok_or(anyhow!("Can't parse 'worklogs' as array"))?
            .into_iter()
            .filter_map(|worklog| {
                self.process_worklog(&issue, worklog, user_name, &start_date, &end_date)
            })
            .collect();

        Ok(worklogs)
    }

    fn process_worklog(
        &self,
        issue: &serde_json::Value,
        worklog: &serde_json::Value,
        user_name: &str,
        start_date: &chrono::NaiveDate,
        end_date: &chrono::NaiveDate,
    ) -> Option<Worklog> {
        let author = worklog["author"]["name"].as_str().unwrap();
        let worklog_started = worklog["started"].as_str().unwrap();
        let worklog_started = NaiveDate::parse_from_str(worklog_started, JIRA_TIME_FORMAT).unwrap();

        if self.author_is(author, user_name)
            && self.date_between(&worklog_started, &start_date, &end_date)
        {
            Some(Worklog::new(
                worklog_started,
                issue["key"].as_str().unwrap(),
                issue["fields"]["summary"].as_str().unwrap(),
                chrono::Duration::seconds(worklog["timeSpentSeconds"].as_u64().unwrap() as i64),
            ))
        } else {
            None
        }
    }

    fn author_is(&self, author: &str, user_name: &str) -> bool {
        author == user_name
    }

    fn date_between(
        &self,
        date: &chrono::NaiveDate,
        start_date: &chrono::NaiveDate,
        end_date: &chrono::NaiveDate,
    ) -> bool {
        date.ge(&start_date) && date.le(&end_date)
    }
}

#[cfg(test)]
mod tests {
    use super::UserWorklogsFetcher;
    use crate::jira::{
        user_worklogs::UserWorklogs, worklog::Worklog, Client, UserWorklogsFetcherTrait,
    };
    use chrono::{Duration, NaiveDate};
    use lazy_static::lazy_static;
    use mockito;

    mod helpers {
        use chrono::NaiveDate;

        pub fn create_issue_entry(id: &str, server_url: &str) -> String {
            format!(
                r###"{{
                    "id": "{id}",
                    "self": "{jira_server_url}/issue/{id}",
                    "key": "{key}",
                    "fields": {{
                        "summary": "{summary}"
                    }}
                }}"###,
                id = id,
                jira_server_url = server_url,
                key = create_issue_key(id),
                summary = create_issue_summary(id),
            )
        }

        pub fn create_issue_key(id: &str) -> String {
            format!("DUMMY-TASK-KEY-{}", id)
        }

        pub fn create_issue_summary(id: &str) -> String {
            format!("ISSUE-SUMMARY-DUMMY-TASK-{}", id)
        }

        pub fn create_worklog_entry(
            author_name: &str,
            started_time: &NaiveDate,
            time_spent: chrono::Duration,
        ) -> String {
            format!(
                r###"{{
                    "author": {{ "name": "{author_name}" }},
                    "started": "{started_time}",
                    "timeSpentSeconds": {time_spent}
                }}"###,
                author_name = author_name,
                started_time = started_time
                    .and_hms_milli(12, 34, 56, 123)
                    .format("%Y-%m-%dT%H:%M:%S%.3f+0300"),
                time_spent = time_spent.num_seconds()
            )
        }

        pub fn create_get_endpoint_mock(
            endpoint: &str,
            body: &str,
            status: usize,
        ) -> mockito::Mock {
            mockito::mock("GET", endpoint)
                .with_status(status)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body(body)
                .create()
        }

        pub fn create_jira_issue_worklogs_endpoint(id: &str) -> String {
            format!("/issue/{}/worklog", id)
        }
    } // mod helpers

    const ISSUE_1_ID: &'static str = "111111";
    const ISSUE_2_ID: &'static str = "222222";

    const USER_NAME: &'static str = "dummy_user_name";
    const ANOTHER_USER_NAME: &'static str = "other_user_name";

    const DUMMY_JIRA_USER: &'static str = "dummy_jira_user";
    const DUMMY_JIRA_PASSWORD: &'static str = "dummy_jira_password";

    lazy_static! {
        static ref DUMMY_TIME_IN_SEC_1: Duration = Duration::seconds(123);
        static ref DUMMY_TIME_IN_SEC_2: Duration = Duration::seconds(360);
        static ref DUMMY_TIME_IN_SEC_3: Duration = Duration::seconds(720);
        static ref START_DATE: NaiveDate = NaiveDate::from_ymd(2022, 09, 10);
        static ref END_DATE: NaiveDate = NaiveDate::from_ymd(2022, 09, 17);
        static ref DATE_IN_SEARCHED_TIME_PERIOD: NaiveDate = NaiveDate::from_ymd(2022, 09, 15);
        static ref DATE_AFTER_END_DATE: NaiveDate = *END_DATE + Duration::days(1);
        static ref DATE_BEFORE_START_DATE: NaiveDate = *START_DATE - Duration::days(1);
    }

    fn create_uut() -> UserWorklogsFetcher<String, Client> {
        let jira_client = Client::new(DUMMY_JIRA_USER, DUMMY_JIRA_PASSWORD);
        UserWorklogsFetcher::new(mockito::server_url(), jira_client)
    }

    #[test]
    fn is_author_returns_true_if_values_are_equal() {
        let uut = create_uut();
        let author = USER_NAME;
        let expected_author = USER_NAME;
        assert_eq!(uut.author_is(author, expected_author), true);
    }

    #[test]
    fn is_author_returns_false_if_values_are_not_equal() {
        let uut = create_uut();
        let author = USER_NAME;
        let expected_author = ANOTHER_USER_NAME;
        assert_eq!(uut.author_is(author, expected_author), false);
    }

    #[test]
    fn date_between_returns_true_if_date_is_equal_to_start_date() {
        let uut = create_uut();
        assert_eq!(
            uut.date_between(&*START_DATE, &*START_DATE, &*END_DATE),
            true
        );
    }

    #[test]
    fn date_between_returns_true_if_date_between_start_and_end_date() {
        let uut = create_uut();
        assert_eq!(
            uut.date_between(&*DATE_IN_SEARCHED_TIME_PERIOD, &*START_DATE, &*END_DATE),
            true
        );
    }

    #[test]
    fn date_between_returns_true_if_date_is_equal_to_end_date() {
        let uut = create_uut();
        assert_eq!(uut.date_between(&*END_DATE, &*START_DATE, &*END_DATE), true);
    }

    #[test]
    fn date_between_returns_false_if_date_is_before_start_date() {
        let uut = create_uut();
        assert_eq!(
            uut.date_between(&*DATE_BEFORE_START_DATE, &*START_DATE, &*END_DATE),
            false
        );
    }

    #[test]
    fn date_between_returns_false_if_date_is_after_end_date() {
        let uut = create_uut();
        assert_eq!(
            uut.date_between(&*DATE_AFTER_END_DATE, &*START_DATE, &*END_DATE),
            false
        );
    }

    #[test]
    fn create_jira_issues_endpoint_returns_correct_url() {
        let uut = create_uut();

        let expected_endpoint_regex = r"/search";
        assert_eq!(
            uut.create_jira_issues_endpoint(USER_NAME, &*START_DATE, &*END_DATE)
                .matches(expected_endpoint_regex)
                .count(),
            1
        );
    }

    #[test]
    fn fetch_issues_returns_ok_status() {
        let uut = create_uut();
        let dummy_issues_response_body = r###"{"key1": "value1", "key2": [123]}"###;
        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            dummy_issues_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        assert!(uut
            .fetch_issues(USER_NAME, &*START_DATE, &*END_DATE)
            .is_ok());
    }

    #[test]
    fn fetch_returns_status_error_if_response_status_of_issues_request_is_not_ok() {
        let uut = create_uut();

        let dummy_issues_response_body = r###"{"key1": "value1", "key2": [123]}"###;
        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            dummy_issues_response_body,
            reqwest::StatusCode::UNAUTHORIZED.as_u16().into(),
        );

        let result = uut.fetch(USER_NAME, *START_DATE, *END_DATE);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<reqwest::Error>().unwrap();
        assert!(error.is_status());
        assert_eq!(error.status(), Some(reqwest::StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn fetch_returns_decode_error_if_cannot_parse_response_body() {
        let uut = create_uut();

        let dummy_issues_response_body = "invalid_json_body";
        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            dummy_issues_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        let result = uut.fetch(USER_NAME, *START_DATE, *END_DATE);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<reqwest::Error>().unwrap();
        assert!(error.is_decode());
    }

    #[test]
    fn fetch_returns_status_error_if_response_status_of_worklog_request_is_not_ok() {
        let uut = create_uut();

        let issues_response_body = format!(
            r###"{{"dummy_field": "values", "issues": [{}, {}]}}"###,
            helpers::create_issue_entry(ISSUE_1_ID, &mockito::server_url()),
            helpers::create_issue_entry(ISSUE_2_ID, &mockito::server_url())
        );
        let issue_1_worklogs_response_body = format!(
            r###"{{"dummy_field": "values", "worklogs": [{}, {}, {}]}}"###,
            helpers::create_worklog_entry(USER_NAME, &START_DATE, *DUMMY_TIME_IN_SEC_1),
            helpers::create_worklog_entry(USER_NAME, &END_DATE, *DUMMY_TIME_IN_SEC_2),
            helpers::create_worklog_entry(
                ANOTHER_USER_NAME,
                &DATE_IN_SEARCHED_TIME_PERIOD,
                *DUMMY_TIME_IN_SEC_3
            ),
        );
        let issue_2_worklogs_response_body = format!(
            r###"{{"dummy_field": "values", "worklogs": [{}, {}, {}]}}"###,
            helpers::create_worklog_entry(USER_NAME, &DATE_BEFORE_START_DATE, *DUMMY_TIME_IN_SEC_1),
            helpers::create_worklog_entry(USER_NAME, &DATE_AFTER_END_DATE, *DUMMY_TIME_IN_SEC_2),
            helpers::create_worklog_entry(
                USER_NAME,
                &DATE_IN_SEARCHED_TIME_PERIOD,
                *DUMMY_TIME_IN_SEC_3
            ),
        );

        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            &issues_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );
        let _issue_1_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_1_ID).as_str(),
            &issue_1_worklogs_response_body,
            reqwest::StatusCode::UNAUTHORIZED.as_u16().into(),
        );
        let _issue_2_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_2_ID).as_str(),
            &issue_2_worklogs_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        let result = uut.fetch(USER_NAME, *START_DATE, *END_DATE);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<reqwest::Error>().unwrap();
        assert!(error.is_status());
        assert_eq!(error.status(), Some(reqwest::StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn fetch_returns_decode_error_if_cannot_parse_worklog_response_body() {
        let uut = create_uut();

        let issues_response_body = format!(
            r###"{{"dummy_field": "values", "issues": [{}, {}]}}"###,
            helpers::create_issue_entry(ISSUE_1_ID, &mockito::server_url()),
            helpers::create_issue_entry(ISSUE_2_ID, &mockito::server_url())
        );
        let issue_1_worklogs_response_body = format!(
            r###"{{"dummy_field": "values", "worklogs": [{}, {}, {}]}}"###,
            helpers::create_worklog_entry(USER_NAME, &START_DATE, *DUMMY_TIME_IN_SEC_1),
            helpers::create_worklog_entry(USER_NAME, &END_DATE, *DUMMY_TIME_IN_SEC_2),
            helpers::create_worklog_entry(
                ANOTHER_USER_NAME,
                &DATE_IN_SEARCHED_TIME_PERIOD,
                *DUMMY_TIME_IN_SEC_3
            ),
        );
        let dummy_worklogs_response_body = "invalid_json_body";
        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            &issues_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );
        let _issue_1_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_1_ID).as_str(),
            &issue_1_worklogs_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );
        let _issue_2_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_2_ID).as_str(),
            &dummy_worklogs_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        let result = uut.fetch(USER_NAME, *START_DATE, *END_DATE);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<reqwest::Error>().unwrap();
        assert!(error.is_decode());
    }

    #[test]
    fn fetch_returns_correct_user_worklogs() {
        let issues_response_body = format!(
            r###"{{"dummy_field": "values", "issues": [{}, {}]}}"###,
            helpers::create_issue_entry(ISSUE_1_ID, &mockito::server_url()),
            helpers::create_issue_entry(ISSUE_2_ID, &mockito::server_url())
        );

        let issue_1_worklogs_response_body = format!(
            r###"{{"dummy_field": "values", "worklogs": [{}, {}, {}]}}"###,
            helpers::create_worklog_entry(USER_NAME, &START_DATE, *DUMMY_TIME_IN_SEC_1),
            helpers::create_worklog_entry(USER_NAME, &END_DATE, *DUMMY_TIME_IN_SEC_2),
            helpers::create_worklog_entry(
                ANOTHER_USER_NAME,
                &DATE_IN_SEARCHED_TIME_PERIOD,
                *DUMMY_TIME_IN_SEC_3
            ),
        );

        let issue_2_worklogs_response_body = format!(
            r###"{{"dummy_field": "values", "worklogs": [{}, {}, {}]}}"###,
            helpers::create_worklog_entry(USER_NAME, &DATE_BEFORE_START_DATE, *DUMMY_TIME_IN_SEC_1),
            helpers::create_worklog_entry(USER_NAME, &DATE_AFTER_END_DATE, *DUMMY_TIME_IN_SEC_2),
            helpers::create_worklog_entry(
                USER_NAME,
                &DATE_IN_SEARCHED_TIME_PERIOD,
                *DUMMY_TIME_IN_SEC_3
            ),
        );

        let uut = create_uut();

        let _issues_endpoint_mock = helpers::create_get_endpoint_mock(
            uut.create_jira_issues_endpoint(USER_NAME, &START_DATE, &END_DATE)
                .as_str(),
            &issues_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        let _issue_1_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_1_ID).as_str(),
            &issue_1_worklogs_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );
        let _issue_2_worklogs_endpoint_mock = helpers::create_get_endpoint_mock(
            helpers::create_jira_issue_worklogs_endpoint(ISSUE_2_ID).as_str(),
            &issue_2_worklogs_response_body,
            reqwest::StatusCode::OK.as_u16().into(),
        );

        let result = uut.fetch(USER_NAME, *START_DATE, *END_DATE);
        assert!(result.is_ok());

        let expected_result = UserWorklogs::new(
            USER_NAME,
            *START_DATE,
            *END_DATE,
            vec![
                Worklog::new(
                    *START_DATE,
                    &helpers::create_issue_key(ISSUE_1_ID),
                    &helpers::create_issue_summary(ISSUE_1_ID),
                    *DUMMY_TIME_IN_SEC_1,
                ),
                Worklog::new(
                    *END_DATE,
                    &helpers::create_issue_key(ISSUE_1_ID),
                    &helpers::create_issue_summary(ISSUE_1_ID),
                    *DUMMY_TIME_IN_SEC_2,
                ),
                Worklog::new(
                    *DATE_IN_SEARCHED_TIME_PERIOD,
                    &helpers::create_issue_key(ISSUE_2_ID),
                    &helpers::create_issue_summary(ISSUE_2_ID),
                    *DUMMY_TIME_IN_SEC_3,
                ),
            ],
        );

        assert_eq!(result.unwrap(), expected_result);
    }
}

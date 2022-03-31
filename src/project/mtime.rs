use ignore::WalkBuilder;
use std::{borrow::Cow, path::Path};
use time::{Duration, OffsetDateTime};

pub(super) fn dir_mtime(path: &Path) -> Option<OffsetDateTime> {
    WalkBuilder::new(path)
        .standard_filters(true)
        .hidden(false)
        .build()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .filter_map(|metadata| metadata.modified().ok())
        .map(OffsetDateTime::from)
        .max()
}

pub(crate) trait HumanReadableElapsed {
    fn human_readable_elapsed(&self) -> Cow<'static, str>;
}

impl HumanReadableElapsed for OffsetDateTime {
    fn human_readable_elapsed(&self) -> Cow<'static, str> {
        human_readable_elapsed(*self, OffsetDateTime::now_utc())
    }
}

fn human_readable_elapsed(since: OffsetDateTime, now: OffsetDateTime) -> Cow<'static, str> {
    static HOUR: i64 = 60;
    static DAY: i64 = 24 * HOUR;
    static WEEK: i64 = 7 * DAY;
    static MONTH: i64 = 28 * DAY; // corrected down so it works for February
    static YEAR: i64 = 365 * DAY;

    let diff: Duration = now - since;

    let was_yesterday = {
        let now_day = now.ordinal();
        if now_day == 1 {
            now.year() - since.year() == 1 && since.ordinal() >= 365
        } else {
            now.year() == since.year() && now.ordinal() - since.ordinal() == 1
        }
    };
    if was_yesterday {
        return "yesterday".into();
    }

    let was_last_week = {
        let (now_year, now_week, _) = now.to_iso_week_date();
        let (since_year, since_week, _) = since.to_iso_week_date();
        if now_week == 1 {
            now_year - since_year == 1 && since_week == 53
        } else {
            now_year == since_year && now_week - since_week == 1
        }
    };
    if was_last_week {
        return "last week".into();
    }

    match diff.whole_minutes() {
        n if n >= 2 * YEAR => format!("{} years ago", n / YEAR).into(),
        n if n >= YEAR => "a year ago".into(),

        n if n >= 2 * MONTH => format!("{} months ago", n / MONTH).into(),
        n if n >= MONTH => "a month ago".into(),

        n if n >= 2 * WEEK => format!("{} weeks ago", n / WEEK).into(),

        n if n >= 2 * DAY => format!("{} days ago", n / DAY).into(),

        n if n >= 2 * HOUR => format!("{} hours ago", n / HOUR).into(),
        n if n >= HOUR => "an hour ago".into(),

        n if n > 9 => format!("{n} minutes ago").into(),
        n if n > 1 => "a few minutes ago".into(),
        n if n == 1 => "a minute ago".into(),
        _ => "just now".into(),
    }
}

#[cfg(test)]
mod test_human_readable_elapsed {
    use time::macros::datetime;

    use super::human_readable_elapsed;

    #[test]
    fn thirty_seconds_ago_is_just_now() {
        let since = datetime!(2022-01-01 0:00:00 UTC);
        let now = datetime!(2022-01-01 0:00:30 UTC);
        assert_eq!(human_readable_elapsed(since, now), "just now");
    }

    #[test]
    fn after_a_minute_it_is_no_longer_now() {
        let since = datetime!(2022-01-01 0:00:00 UTC);
        let now = datetime!(2022-01-01 0:01:30 UTC);
        assert_eq!(human_readable_elapsed(since, now), "a minute ago");
    }

    #[test]
    fn five_minutes_is_a_few_minutes() {
        let since = datetime!(2022-01-01 0:00 UTC);
        let now = datetime!(2022-01-01 0:05 UTC);
        assert_eq!(human_readable_elapsed(since, now), "a few minutes ago");
    }

    #[test]
    fn less_than_an_hour_tells_the_minutes() {
        let since = datetime!(2022-01-01 0:00 UTC);
        let now = datetime!(2022-01-01 0:59 UTC);
        assert_eq!(human_readable_elapsed(since, now), "59 minutes ago");
    }

    #[test]
    fn less_than_a_day_tells_the_hours_except_when_that_was_yesterday_the_year_before() {
        let now = datetime!(2022-01-01 15:00 UTC);

        let earlier_today = datetime!(2022-01-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(earlier_today, now), "15 hours ago");

        let yesterday = datetime!(2021-12-31 23:59 UTC);
        assert_eq!(human_readable_elapsed(yesterday, now), "yesterday");
    }

    #[test]
    fn less_than_a_day_tells_the_hours_except_when_that_was_yesterday_this_year() {
        let now = datetime!(2022-02-01 15:00 UTC);

        let earlier_today = datetime!(2022-02-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(earlier_today, now), "15 hours ago");

        let yesterday = datetime!(2022-01-31 23:59 UTC);
        assert_eq!(human_readable_elapsed(yesterday, now), "yesterday");
    }

    #[test]
    fn less_than_a_week_tells_the_days_except_when_that_was_last_week() {
        let now_thursday = datetime!(2022-03-31 15:00 UTC);

        let monday = datetime!(2022-03-28 0:00 UTC);
        assert_eq!(human_readable_elapsed(monday, now_thursday), "3 days ago");

        let friday_last_week = datetime!(2022-03-25 0:00 UTC);
        assert_eq!(
            human_readable_elapsed(friday_last_week, now_thursday),
            "last week"
        );
    }

    #[test]
    fn three_weeks_ago_works_across_a_month_boundary() {
        let since = datetime!(2022-01-05 0:00 UTC);
        let now = datetime!(2022-02-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(since, now), "3 weeks ago");
    }

    #[test]
    fn a_month_ago_also_works_with_february() {
        let since = datetime!(2022-02-01 0:00 UTC);
        let now = datetime!(2022-03-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(since, now), "a month ago");
    }

    #[test]
    fn less_than_a_year_tells_the_months_even_if_that_was_last_year() {
        let since = datetime!(2021-02-01 0:00 UTC);
        let now = datetime!(2022-01-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(since, now), "11 months ago");
    }

    #[test]
    fn twelve_months_means_a_year_has_passed() {
        let since = datetime!(2021-02-01 0:00 UTC);
        let now = datetime!(2022-02-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(since, now), "a year ago");
    }

    #[test]
    fn with_more_than_one_year_we_see_the_plural() {
        let since = datetime!(2020-02-01 0:00 UTC);
        let now = datetime!(2022-02-01 0:00 UTC);
        assert_eq!(human_readable_elapsed(since, now), "2 years ago");
    }
}

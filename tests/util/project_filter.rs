use chrono::Duration;
use makeclean::ProjectFilter;

pub fn noop_project_filter() -> ProjectFilter {
    ProjectFilter {
        min_age: Duration::days(0),
        status: makeclean::ProjectStatus::Any,
    }
}

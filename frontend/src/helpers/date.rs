use chrono::{Duration, Local, NaiveDateTime};

pub const DATE_FORMAT: &str = "%y/%m/%d %H:%M:%S";
pub const EXPIRATION_WARNING_DAYS: i64 = 2;

#[derive(Debug, PartialEq, Clone)]
pub enum DateStatus {
    Valid,
    Expired,
    AboutToExpire,
    Invalid,
}

pub fn format_date(date: Option<NaiveDateTime>) -> String {
    match date {
        Some(dt) => dt.format(DATE_FORMAT).to_string(),
        None => String::new(),
    }
}

pub fn validate_date(date: Option<NaiveDateTime>) -> DateStatus {
    match date {
        Some(date_time) => {
            let current_date = Local::now().naive_local();
            let warning_date = current_date + Duration::days(EXPIRATION_WARNING_DAYS);

            if date_time < current_date {
                DateStatus::Expired
            } else if date_time <= warning_date {
                DateStatus::AboutToExpire
            } else {
                DateStatus::Valid
            }
        }
        None => DateStatus::Invalid,
    }
}

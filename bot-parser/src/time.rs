use chrono::{
    DateTime, Datelike, Duration, FixedOffset, Local, NaiveDateTime, NaiveTime, TimeZone, Utc,
};
use std::cell::LazyCell;

use crate::error::Result;

const ASIA_TOKYO: LazyCell<FixedOffset> =
    LazyCell::new(|| chrono::FixedOffset::from_offset(&FixedOffset::east_opt(9 * 3600).unwrap()));

pub(crate) fn parse_time(value: &str) -> Result<DateTime<FixedOffset>> {
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        Ok((*ASIA_TOKYO).from_utc_datetime(&datetime))
    } else if let Ok(parsed_time) = NaiveTime::parse_from_str(value, "%H:%M") {
        // 時刻のみ指定されている場合

        let datetime =
            NaiveDateTime::new(Utc::now().date_naive(), parsed_time - Duration::hours(9));
        // HACK:
        // 正しく変換できない。DateはUTCだけど、parsed_timeはユーザーが書いた、UTC+9だから

        Ok((*ASIA_TOKYO).from_utc_datetime(&datetime))
    } else {
        Err(crate::error::Error::TimeFormat(format!("{value}")))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_time() -> Result<(), Box<dyn std::error::Error>> {
        let _ = super::parse_time("16:07").unwrap();
        let _ = super::parse_time("2024-12-04T16:07").unwrap();
        let _ = super::parse_time("23:59").unwrap();
        let _ = super::parse_time("2000-1-1T01:01").unwrap();
        Ok(())
    }
}

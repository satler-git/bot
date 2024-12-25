use crate::error::Result;
use chrono::{Duration, NaiveDateTime, NaiveTime, Utc};

pub(crate) fn parse_time(value: &str) -> Result<NaiveDateTime> {
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        Ok(datetime)
    } else if let Ok(parsed_time) = NaiveTime::parse_from_str(value, "%H:%M") {
        // 時刻のみ指定されている場合

        Ok(NaiveDateTime::new(Utc::now().date_naive(), parsed_time))
        // HACK:
        // 正しく変換できない。DateはUTCだけど、parsed_timeはユーザーが書いた、UTC+9だから
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

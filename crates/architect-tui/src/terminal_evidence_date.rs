use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::terminal_evidence::sanitize_note;

pub(crate) fn collected_at_value(value: Option<&str>) -> Result<String> {
    if let Some(value) = value
        .map(sanitize_note)
        .filter(|value| !value.trim().is_empty())
    {
        validate_collected_at_date(&value)?;
        return Ok(value);
    }
    current_utc_date()
}

fn current_utc_date() -> Result<String> {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            anyhow::anyhow!(
                "system clock is before the Unix epoch; pass --collected-at YYYY-MM-DD for verified external evidence"
            )
        })?
        .as_secs()
        / 86_400;
    Ok(unix_days_to_date(days))
}

fn validate_collected_at_date(value: &str) -> Result<()> {
    let Some((year, month, day)) = parse_collected_at_date(value) else {
        anyhow::bail!("--collected-at must use YYYY-MM-DD")
    };
    if year < 1970 || month == 0 || month > 12 {
        anyhow::bail!("--collected-at must use YYYY-MM-DD")
    }
    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        anyhow::bail!("--collected-at must use YYYY-MM-DD")
    }
    Ok(())
}

fn parse_collected_at_date(value: &str) -> Option<(u32, u32, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return None;
    }
    let year = value[0..4].parse().ok()?;
    let month = value[5..7].parse().ok()?;
    let day = value[8..10].parse().ok()?;
    Some((year, month, day))
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

pub(crate) fn unix_days_to_date(days: u64) -> String {
    let mut z = days as i64;
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    format!("{year:04}-{month:02}-{day:02}")
}

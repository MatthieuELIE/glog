const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

pub fn relative(now: i64, time: i64) -> String {
    let elapsed = now.saturating_sub(time);
    if elapsed < MINUTE {
        return "now".to_string();
    }
    if elapsed < HOUR {
        return format!("{}m", elapsed / MINUTE);
    }
    if elapsed < DAY {
        return format!("{}h", elapsed / HOUR);
    }
    if elapsed < MONTH {
        return format!("{}d", elapsed / DAY);
    }
    if elapsed < YEAR {
        return format!("{}mo", elapsed / MONTH);
    }

    format!("{}y", elapsed / YEAR)
}

#[cfg(test)]
mod tests;

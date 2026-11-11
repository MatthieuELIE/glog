use super::relative;

const NOW: i64 = 2_000_000_000;
const MINUTE: i64 = 60;
const HOUR: i64 = 3_600;
const DAY: i64 = 86_400;

fn ago(seconds: i64) -> String {
    relative(NOW, NOW - seconds)
}

#[test]
fn shows_now_under_a_minute() {
    assert_eq!(ago(0), "now");
    assert_eq!(ago(59), "now");
}

#[test]
fn shows_minutes_from_sixty_seconds() {
    assert_eq!(ago(MINUTE), "1m");
    assert_eq!(ago(42 * MINUTE), "42m");
    assert_eq!(ago(HOUR - 1), "59m");
}

#[test]
fn shows_hours_from_one_hour() {
    assert_eq!(ago(HOUR), "1h");
    assert_eq!(ago(5 * HOUR), "5h");
    assert_eq!(ago(DAY - 1), "23h");
}

#[test]
fn shows_days_from_one_day() {
    assert_eq!(ago(DAY), "1d");
    assert_eq!(ago(12 * DAY), "12d");
    assert_eq!(ago(30 * DAY - 1), "29d");
}

#[test]
fn shows_months_of_thirty_days_from_thirty_days() {
    assert_eq!(ago(30 * DAY), "1mo");
    assert_eq!(ago(90 * DAY), "3mo");
    assert_eq!(ago(365 * DAY - 1), "12mo");
}

#[test]
fn shows_years_of_365_days_from_365_days() {
    assert_eq!(ago(365 * DAY), "1y");
    assert_eq!(ago(2 * 365 * DAY), "2y");
}

#[test]
fn shows_years_for_the_smallest_timestamp_without_overflowing() {
    assert!(relative(NOW, i64::MIN).ends_with('y'));
}

#[test]
fn shows_now_for_a_timestamp_in_the_future() {
    assert_eq!(ago(-DAY), "now");
}

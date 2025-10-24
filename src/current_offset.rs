use crate::parameters::{SearchDirection, Timestamp};

use chrono::Datelike;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use mocks::Utc;

pub fn get_offset_from_timestamp(
    timestamp: Timestamp,
    direction: SearchDirection,
    max_duration_hours: i32,
) -> i32 {
    let final_day = if direction == SearchDirection::Forward {
        timestamp + chrono::Duration::hours(max_duration_hours.into())
    } else {
        timestamp
    }
    .num_days_from_ce();
    let today = Utc::now().num_days_from_ce();
    final_day - today
}

pub fn get_offset_from_now(direction: SearchDirection, max_duration_hours: i32) -> i32 {
    let now = Utc::now();
    if direction == SearchDirection::Forward {
        (now + chrono::Duration::hours(max_duration_hours.into())).num_days_from_ce()
            - now.num_days_from_ce()
    } else {
        0i32
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    use chrono::DateTime;

    #[test]
    fn getOffsetFromTimestamp_forwardSeachEndingOnSameDay_getOffset0() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-16 11:59:59+00:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Forward, 12);

        assert_eq!(curret_offset, 0i32);
    }

    #[test]
    fn getOffsetFromTimestamp_forwardSeachEndingOnNextDay_getOffset1() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-16 12:00:00+00:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Forward, 12);

        assert_eq!(curret_offset, 1i32);
    }

    #[test]
    fn getOffsetFromTimestamp_forwardSeachEndingIn5Days_getOffset5() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-20 12:00:00+02:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Forward, 24);

        assert_eq!(curret_offset, 5i32);
    }

    #[test]
    fn getOffsetFromTimestamp_forwardSeachStarting2DaysAgo_getOffsetMinus1() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-14 12:00:00+02:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Forward, 24);

        assert_eq!(curret_offset, -1i32);
    }

    #[test]
    fn getOffsetFromTimestamp_backwardSeachEndingOnSameDay_getOffset0() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-16 23:59:59+00:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Backward, 12);

        assert_eq!(curret_offset, 0i32);
    }

    #[test]
    fn getOffsetFromTimestamp_backwardSeachEndingOnNextDay_getOffset1() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-17 00:00:00+00:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Backward, 12);

        assert_eq!(curret_offset, 1i32);
    }

    #[test]
    fn getOffsetFromTimestamp_backwardSeachEndingIn5Days_getOffset5() {
        let timestamp = DateTime::parse_from_rfc3339("2025-10-21 12:00:00+00:00")
            .unwrap()
            .to_utc();

        let curret_offset = get_offset_from_timestamp(timestamp, SearchDirection::Backward, 24);

        assert_eq!(curret_offset, 5i32);
    }

    #[test]
    fn getOffsetFromNow_forwardSeachEndingOnSameDay_getOffset0() {
        let curret_offset = get_offset_from_now(SearchDirection::Forward, 16);

        assert_eq!(curret_offset, 0i32);
    }

    #[test]
    fn getOffsetFromNow_forwardSeachEndingOnNextDay_getOffset1() {
        let curret_offset = get_offset_from_now(SearchDirection::Forward, 17);

        assert_eq!(curret_offset, 1i32);
    }

    #[test]
    fn getOffsetFromNow_forwardSeachEndingIn5Days_getOffset5() {
        let curret_offset = get_offset_from_now(SearchDirection::Forward, 24 * 5);

        assert_eq!(curret_offset, 5i32);
    }

    #[test]
    fn getOffsetFromNow_backwardSeachAnyDuration_getOffset0() {
        let curret_offset = get_offset_from_now(SearchDirection::Backward, 9000);

        assert_eq!(curret_offset, 0i32);
    }
}

#[cfg(test)]
mod mocks {
    use chrono::DateTime;

    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime<chrono::Utc> {
            DateTime::parse_from_rfc3339("2025-10-16T09:27:00+02:00")
                .unwrap()
                .to_utc()
        }
    }
}

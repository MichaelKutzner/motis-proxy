use std::collections::HashMap;

use chrono::{DateTime, Datelike, Utc};
use hyper::{Request, body::Incoming};
use url::Url;

pub fn get_current_day_offset(req: &Request<Incoming>) -> Option<i32> {
    match parse_query(req.uri().query()) {
        Some(parameters) => {
            if parameters.contains_key("pageCursor") {
                // No restrictions for paged queries
                None
            } else {
                parse_time_parameter(parameters)
            }
        }
        // No parameters: Assume now
        None => Some(0i32),
    }
}

fn parse_query(query: Option<&str>) -> Option<HashMap<String, String>> {
    query.and_then(|query| {
        let url = format!("http://localhost/?{}", query);
        Url::parse(url.as_str())
            .and_then(|parsed| Ok(parsed.query_pairs().into_owned().collect()))
            .ok()
    })
}

fn parse_time_parameter(parameters: HashMap<String, String>) -> Option<i32> {
    match parameters.get("time") {
        Some(time) => parse_day_offset(time),
        // No 'time' parameter: Assume now
        None => Some(0i32),
    }
}

fn parse_day_offset(time: &String) -> Option<i32> {
    match parse_day(time) {
        Some(day) => {
            let today = Utc::now().num_days_from_ce();
            // println!("Offset day: {}", day - today);
            Some(day - today)
        }
        // Parsing failed: Cannot compute days_from_now
        None => None,
    }
}

fn parse_day(time: &String) -> Option<i32> {
    DateTime::parse_from_rfc3339(time)
        .ok()
        .and_then(|ts| Some(ts.to_utc()))
        .or_else(|| {
            // Fallback: Use unixtime
            time.parse::<i64>()
                .ok()
                .and_then(|unixtime| DateTime::from_timestamp_secs(unixtime))
        })
        .and_then(|ts| Some(ts.num_days_from_ce()))
}

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use hyper::{Request, body::Incoming};
use url::Url;

pub enum SearchParameters {
    Timestamp {
        timestamp: Timestamp,
        direction: SearchDirection,
    }, // 'time' set
    Now {
        direction: SearchDirection,
    }, // 'time' not set
    Unrestricted, // Using paged requests, i.e. use largest instance
    None,         // No query parameters, e.g. static data
}

pub type Timestamp = DateTime<Utc>;

#[derive(Debug, PartialEq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

pub fn parse_parameters(req: &Request<Incoming>) -> SearchParameters {
    match parse_query(req.uri().query()) {
        Some(parameters) => {
            if parameters.contains_key("pageCursor") {
                // No restrictions for paged queries
                SearchParameters::Unrestricted
            } else {
                let direction = parse_direction(&parameters);
                parse_time_parameter(parameters, direction)
            }
        }
        // No query parameters; Possibly a static file
        None => SearchParameters::None,
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

fn parse_direction(parameters: &HashMap<String, String>) -> SearchDirection {
    if parameters
        .get("arriveBy")
        .and_then(|arr| arr.parse::<bool>().ok())
        .unwrap_or(false)
    {
        SearchDirection::Backward
    } else {
        SearchDirection::Forward
    }
}

fn parse_time_parameter(
    parameters: HashMap<String, String>,
    direction: SearchDirection,
) -> SearchParameters {
    match parameters.get("time") {
        Some(time) => parse_day_offset(time, direction),
        // No 'time' parameter: Assume now
        None => SearchParameters::Now { direction },
    }
}

fn parse_day_offset(time: &String, direction: SearchDirection) -> SearchParameters {
    match parse_timestamp(time) {
        Some(timestamp) => SearchParameters::Timestamp {
            timestamp,
            direction,
        },
        // Parsing failed: Use unrestricted search
        None => SearchParameters::Unrestricted,
    }
}

fn parse_timestamp(time: &String) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(time)
        .ok()
        .and_then(|ts| Some(ts.to_utc()))
        .or_else(|| {
            // Fallback: Use unixtime
            time.parse::<i64>()
                .ok()
                .and_then(|unixtime| DateTime::from_timestamp_secs(unixtime))
        })
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn parseDirection_arriveByIsTrue_getDirectionBackward() {
        let parameters: HashMap<String, String> =
            HashMap::from([("arriveBy".into(), "true".into())]);

        let direction = parse_direction(&parameters);

        assert_eq!(direction, SearchDirection::Backward);
    }

    #[test]
    fn parseDirection_arriveByIsFalse_getDirectionForward() {
        let parameters: HashMap<String, String> =
            HashMap::from([("arriveBy".into(), "false".into())]);

        let direction = parse_direction(&parameters);

        assert_eq!(direction, SearchDirection::Forward);
    }

    #[test]
    fn parseDirection_missingArriveBy_getDirectionForward() {
        let parameters: HashMap<String, String> = HashMap::new();

        let direction = parse_direction(&parameters);

        assert_eq!(direction, SearchDirection::Forward);
    }
}

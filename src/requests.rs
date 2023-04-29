use chrono::prelude::*;
use lazy_regex::regex_captures;
use crate::orders::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum RequestType {
    Create,
    Replace,
    Cancel
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    timestamp: DateTime<FixedOffset>,
    order_id: u64,
    request_type: RequestType,
    content: OrderContent
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl TryFrom<&str> for Request {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        let (_, cap_timestamp, cap_order_id, cap_request_type, cap_content) = regex_captures!(r#"\[(.*?)\] ([0-9]*) (.*?) \"(.*?)\""#, value).unwrap();
        let timestamp = DateTime::parse_from_rfc2822(cap_timestamp)
        .map_err(|_| "failed to parse timestamp into rfc2822 date".to_owned())?;

        let order_id = cap_order_id.parse().map_err(|_| format!("failed to parse order_id into u64")).unwrap();
        
        let request_type = cap_request_type.try_into().map_err(|_| format!("failed to parse request_type")).unwrap();
        
        let content = cap_content.try_into().map_err(|_| format!("failed to parse order contents")).unwrap();
        Ok(Self {
            timestamp,
            order_id,
            request_type,
            content
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{requests::{RequestType, Request}, orders::OrderContent};

    #[test]
    fn should_parse_valid_string() {
        let log_line =
            "[Sat, 29 Apr 2023 23:16:09 GMT] 7 CREATE \"LIMIT_ORDER_BUY 200 100\"";

        let expected = Request {
            timestamp: DateTime::parse_from_rfc2822("Sat, 29 Apr 2023 23:16:09 GMT").unwrap(),
            order_id: 7,
            request_type: RequestType::Create,
            content: OrderContent::LimitOrderBuy { price: 200, quantity: 100 }
            };

        let result: Result<Request, _> = log_line.try_into();

        assert_eq!(result.unwrap(), expected);
    }
}

impl TryFrom<&str> for RequestType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CREATE" => Ok(RequestType::Create),
            "REPLACE" => Ok(RequestType::Replace),
            "CANCEL" => Ok(RequestType::Cancel),
            _ => Err(format!("Unknown request type: {}", value))
        }
    }
}
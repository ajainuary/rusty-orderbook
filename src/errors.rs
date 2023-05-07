use std::fmt;

pub type OrderbookResult<T> = std::result::Result<T, OrderbookProcessingError>;
pub type OrderResult<T> = std::result::Result<T, OrderError>;

#[derive(Debug, Clone)]
pub struct OrderbookProcessingError;

impl fmt::Display for OrderbookProcessingError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There was an error processing the orderbook.")
    }
}
#[derive(Debug, Clone)]
pub struct OrderError;

impl fmt::Display for OrderError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There was an error processing the order.")
    }
}
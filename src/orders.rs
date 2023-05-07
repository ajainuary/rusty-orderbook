use lazy_regex::regex_captures;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderContent {
    LimitOrderBuy {
        price: u64,
        quantity: u64
    },
    LimitOrderSell {
        price: u64,
        quantity: u64
    },
    Empty
}

impl TryFrom<&str> for OrderContent {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once(" ") {
            Some((order_type, rest)) => {
                match order_type {
                    "LIMIT_ORDER_BUY" => {
                        let (_, price, quantity) = regex_captures!(r#"([0-9]+) ([0-9]+)"#, rest).unwrap();
                        println!("Price: {}, Quantity: {}", price, quantity);
                        Ok(OrderContent::LimitOrderBuy {
                            price: price.parse().unwrap(),
                            quantity: quantity.parse().unwrap()
                        })
                    },
                    "LIMIT_ORDER_SELL" => {
                        let (_, price, quantity) = regex_captures!(r#"([0-9]+) ([0-9]+)"#, rest).unwrap();
                        Ok(OrderContent::LimitOrderSell {
                            price: price.parse().unwrap(),
                            quantity: quantity.parse().unwrap()
                        })
                    },
                    _ => Err(format!("Unkown order type")),
                }
            }
            None => {
                match value {
                    "EMPTY" => Ok(OrderContent::Empty),
                    _ => Err(format!("Unkown order type"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_create_limit_order_buy_content() {
        let order_contents = "LIMIT_ORDER_BUY 100 200";

        let expected = OrderContent::LimitOrderBuy {
            price: 100,
            quantity: 200
        };

        let result: Result<OrderContent, _> = order_contents.try_into();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_create_limit_order_sell_content() {
        let order_contents = "LIMIT_ORDER_SELL 100 200";

        let expected = OrderContent::LimitOrderSell {
            price: 100,
            quantity: 200
        };

        let result: Result<OrderContent, _> = order_contents.try_into();
        assert_eq!(result.unwrap(), expected);
    }
}
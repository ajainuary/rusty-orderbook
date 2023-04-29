use lazy_regex::regex_captures;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OrderContent {
    LimitOrderBuy {
        price: u32,
        quantity: u32
    },
    LimitOrderSell {
        price: u32,
        quantity: u32
    }
}

pub  struct Order {
    order_id: u64,
    content: OrderContent
}

impl TryFrom<&str> for OrderContent {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (order_type, rest) = value
            .split_once(" ")
            .ok_or(format!("Unable to parse order contents: {}", value)).unwrap();
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

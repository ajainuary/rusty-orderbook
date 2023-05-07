use std::collections::LinkedList;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::ops::DerefMut;
use dashmap::DashMap;

use crate::errors::*;
use crate::requests::*;
use crate::orders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LimitType {
    Bid,
    Ask,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Limit {
    price: u64,
    limit_type: LimitType,
    orders: LinkedList<u64>,
}

pub fn process_requests(request_list: &mut [Request]) -> OrderbookResult<()> {
    let orders: DashMap<u64, OrderContent> = DashMap::new();
    let tick_map: DashMap<u64, Limit> = DashMap::new();
    let mut limits_bid: BinaryHeap<u64> = BinaryHeap::new();
    let mut limits_ask: BinaryHeap<Reverse<u64>> = BinaryHeap::new();
    let mut count = 1;
    for request in request_list.iter_mut() {
        println!("{:?}", request);
        match request.request_type {
            RequestType::Create =>
                new_order(&orders, &tick_map, &mut limits_bid, &mut limits_ask, request).unwrap(),
            _ => println!("error"),
        }
        println!("After request #: {}", count);
        count += 1;
        print_orderbook(&tick_map, &orders, &mut limits_bid, &mut limits_ask);
    }
    Ok(())
}
fn print_orderbook(
    tick_map: &DashMap<u64, Limit>,
    orders: &DashMap<u64, OrderContent>,
    limits_bid: &mut BinaryHeap<u64>,
    limits_ask: &mut BinaryHeap<Reverse<u64>>
) {
    for tick in tick_map.iter() {
        println!("{:?}@{}", tick.value().limit_type, tick.value().price);
        for order in &tick.value().orders {
            println!("{}:{:?}", order, *orders.get(order).unwrap());
        }
    }
    if limits_bid.peek().is_some() {
        println!("Best bid: {}", limits_bid.peek().unwrap());
    }
    if limits_ask.peek().is_some() {
        println!("Best ask: {}", limits_ask.peek().unwrap().0)
    }
}

fn new_order(
    orders: &DashMap<u64, OrderContent>,
    tick_map: &DashMap<u64, Limit>,
    limits_bid: &mut BinaryHeap<u64>,
    limits_ask: &mut BinaryHeap<Reverse<u64>>,
    request: &mut Request
) -> OrderbookResult<()> {
    if orders.contains_key(&request.order_id) {
        println!("Duplicate create request for order id: {}", request.order_id);
        return Ok(());
    }
    //First execute the order
    println!("Executing request {:?}", request.content);
    match execute_order(&mut request.content, orders, tick_map, limits_bid, limits_ask) {
        Some(OrderContent::LimitOrderBuy { price, quantity }) => {
            if tick_map.contains_key(&price) {
                // Limit already exists, append order
                tick_map.get_mut(&price).unwrap().orders.push_back(request.order_id);
            } else {
                tick_map.insert(*price, Limit {
                    price: *price,
                    limit_type: LimitType::Bid,
                    orders: LinkedList::from([request.order_id]),
                });
                limits_bid.push(*price);
            }
            orders.insert(request.order_id, request.content);
            Ok(())
        }
        Some(OrderContent::LimitOrderSell { price, quantity }) => {
            if tick_map.contains_key(&price) {
                // Limit already exists, append order
                tick_map.get_mut(&price).unwrap().orders.push_back(request.order_id);
            } else {
                tick_map.insert(*price, Limit {
                    price: *price,
                    limit_type: LimitType::Ask,
                    orders: LinkedList::from([request.order_id]),
                });
                limits_ask.push(Reverse(*price));
            }
            orders.insert(request.order_id, request.content);
            Ok(())
        }
        None => Ok(()),
    }
}

fn execute_order<'a>(
    order_content: &'a mut OrderContent,
    orders: &DashMap<u64, OrderContent>,
    tick_map: &DashMap<u64, Limit>,
    limits_bid: &mut BinaryHeap<u64>,
    limits_ask: &mut BinaryHeap<Reverse<u64>>
) -> Option<&'a mut OrderContent> {
    while (
        match &order_content {
            OrderContent::LimitOrderBuy { price, quantity } =>
                limits_ask.peek().is_some() && *price >= limits_ask.peek()?.0 && *quantity > 0,
            OrderContent::LimitOrderSell { price, quantity } =>
                limits_bid.peek().is_some() && *price <= *limits_bid.peek()? && *quantity > 0,
        }
    ) {
        let trade_price = match &order_content {
            OrderContent::LimitOrderBuy { price: _, quantity: _ } => (*limits_ask.peek()?).0,
            OrderContent::LimitOrderSell { price: _, quantity: _ } => *limits_bid.peek()?,
        };
        {
            let mut matched_limit = tick_map.get_mut(&trade_price).unwrap();
            match (&mut *order_content, matched_limit.limit_type) {
                | (OrderContent::LimitOrderBuy { price: _, ref mut quantity }, LimitType::Ask)
                | (OrderContent::LimitOrderSell { price: _, ref mut quantity }, LimitType::Bid) => {
                    while *quantity > 0 && matched_limit.orders.front().is_some() {
                        let matched_order_id = matched_limit.orders.front().unwrap();
                        match
                            orders.remove_if(matched_order_id, |_, &matched_order| {
                                match matched_order {
                                    | OrderContent::LimitOrderSell {
                                          price: _,
                                          quantity: matched_quantity,
                                      }
                                    | OrderContent::LimitOrderBuy {
                                          price: _,
                                          quantity: matched_quantity,
                                      } => matched_quantity <= *quantity,
                                }
                            })
                        {
                            Some((_, executed_order)) => {
                                match executed_order {
                                    | OrderContent::LimitOrderSell {
                                          price: _,
                                          quantity: matched_quantity,
                                      }
                                    | OrderContent::LimitOrderBuy {
                                          price: _,
                                          quantity: matched_quantity,
                                      } => {
                                        *quantity -= matched_quantity;
                                        matched_limit.orders.pop_front();
                                    }
                                    _ => {}
                                }
                            }
                            None => {
                                match orders.get_mut(matched_order_id)?.deref_mut() {
                                    | OrderContent::LimitOrderSell {
                                          price: _,
                                          quantity: matched_quantity,
                                      }
                                    | OrderContent::LimitOrderBuy {
                                          price: _,
                                          quantity: matched_quantity,
                                      } => {
                                        *matched_quantity -= *quantity;
                                        *quantity = 0;
                                        return None;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        tick_map.remove(&trade_price);
        match &order_content {
            OrderContent::LimitOrderBuy { price: _, quantity: _ } => limits_ask.pop()?.0,
            OrderContent::LimitOrderSell { price: _, quantity: _ } => limits_bid.pop()?,
        };
    }
    Some(order_content)
}
use std::collections::LinkedList;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use dashmap::DashMap;

use crate::errors::*;
use crate::requests::*;
use crate::orders::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    match request.content {
        OrderContent::LimitOrderBuy { price: buy_price, quantity: _ } => {
            execute_bid(&mut request.content, orders, tick_map, limits_ask);
            match request.content {
                OrderContent::LimitOrderBuy { price: _, quantity } => if quantity == 0 {
                    return Ok(());
                }
                _ => {}
            }
            if tick_map.contains_key(&buy_price) {
                // Limit already exists, append order
                tick_map.get_mut(&buy_price).unwrap().orders.push_back(request.order_id);
            } else {
                tick_map.insert(buy_price, Limit {
                    price: buy_price,
                    limit_type: LimitType::Bid,
                    orders: LinkedList::from([request.order_id]),
                });
                limits_bid.push(buy_price);
            }
        }
        OrderContent::LimitOrderSell { price: sell_price, quantity: _ } => {
            execute_ask(&mut request.content, orders, tick_map, limits_bid);
            match request.content {
                OrderContent::LimitOrderSell { price: _, quantity } => if quantity == 0 {
                    return Ok(());
                }
                _ => {}
            }
            if tick_map.contains_key(&sell_price) {
                // Limit already exists, append order
                tick_map.get_mut(&sell_price).unwrap().orders.push_back(request.order_id);
            } else {
                tick_map.insert(sell_price, Limit {
                    price: sell_price,
                    limit_type: LimitType::Bid,
                    orders: LinkedList::from([request.order_id]),
                });
                limits_ask.push(Reverse(sell_price));
            }
        }
    }
    orders.insert(request.order_id, request.content);
    Ok(())
}

fn execute_bid(
    order_content: &mut OrderContent,
    orders: &DashMap<u64, OrderContent>,
    tick_map: &DashMap<u64, Limit>,
    limits_ask: &mut BinaryHeap<Reverse<u64>>
) {
    match order_content {
        OrderContent::LimitOrderBuy { price, quantity } => {
            while
                limits_ask.peek().is_some() &&
                price >= &mut limits_ask.peek_mut().unwrap().0 &&
                *quantity > 0
            {
                let trade_price = &limits_ask.peek().unwrap().0;
                {
                    let mut matched_limit = tick_map.get_mut(trade_price).unwrap();
                    while quantity > &mut 0 && matched_limit.orders.front().is_some() {
                        let matched_order_id = matched_limit.orders.front().unwrap();
                        match
                            orders.remove_if(matched_order_id, |_, matched_order| {
                                match matched_order {
                                    OrderContent::LimitOrderSell {
                                        price: _,
                                        quantity: mut matched_quantity,
                                    } => matched_quantity <= *quantity,
                                    _ => false,
                                }
                            })
                        {
                            Some((_, executed_order)) => {
                                match executed_order {
                                    OrderContent::LimitOrderSell {
                                        price: _,
                                        quantity: mut matched_quantity,
                                    } => {
                                        *quantity -= matched_quantity;
                                        matched_limit.orders.pop_front();
                                    }
                                    _ => {}
                                }
                            }
                            None => {
                                match *orders.get_mut(matched_order_id).unwrap() {
                                    OrderContent::LimitOrderSell {
                                        price: _,
                                        quantity: ref mut matched_quantity,
                                    } => {
                                        *matched_quantity -= *quantity;
                                        *quantity = 0;
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                tick_map.remove(&limits_ask.pop().unwrap().0);
            }
        }
        _ => {}
    }
}

fn execute_ask(
    order_content: &mut OrderContent,
    orders: &DashMap<u64, OrderContent>,
    tick_map: &DashMap<u64, Limit>,
    limits_bid: &mut BinaryHeap<u64>
) {
    match order_content {
        OrderContent::LimitOrderSell { price, quantity } => {
            while
                limits_bid.peek().is_some() &&
                price <= &mut limits_bid.peek_mut().unwrap() &&
                *quantity > 0
            {
                let trade_price = limits_bid.peek().unwrap();
                {
                    let mut matched_limit = tick_map.get_mut(trade_price).unwrap();
                    while quantity > &mut 0 && matched_limit.orders.front().is_some() {
                        let matched_order_id = matched_limit.orders.front().unwrap();
                        match
                            orders.remove_if(matched_order_id, |_, matched_order| {
                                match matched_order {
                                    OrderContent::LimitOrderBuy {
                                        price: _,
                                        quantity: mut matched_quantity,
                                    } => matched_quantity <= *quantity,
                                    _ => false,
                                }
                            })
                        {
                            Some((_, executed_order)) => {
                                match executed_order {
                                    OrderContent::LimitOrderBuy {
                                        price: _,
                                        quantity: mut matched_quantity,
                                    } => {
                                        *quantity -= matched_quantity;
                                        matched_limit.orders.pop_front();
                                    }
                                    _ => {}
                                }
                            }
                            None => {
                                match *orders.get_mut(matched_order_id).unwrap() {
                                    OrderContent::LimitOrderBuy {
                                        price: _,
                                        quantity: ref mut matched_quantity,
                                    } => {
                                        *matched_quantity -= *quantity;
                                        *quantity = 0;
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                tick_map.remove(&limits_bid.pop().unwrap());
            }
        }
        _ => {}
    }
}
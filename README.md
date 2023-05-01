# rusty-orderbook
An attempt to build an orderbook exchange in Rust

# Usage
1. Make sure you have installed the rust environment. (Further instructions at [rustup.rs](https://rustup.rs/))
2. To build the release / optimized binary: `cargo build --release`
3. Execute the binary: `./target/release/rusty-orderbook --input <INPUT_LOGS>` (You can try some sample logs in `sample_logs` folder

## Testing
Run the `cargo test` command to run the unit tests.

# Theoretical Analysis
The orderbook is implemented with the help of maps for mapping the order ids and map order lists to the tick they are part of and heaps for determining the best bid and best ask at any point of time in O(1).

## Best Bid and Best Ask
O(1) using heaps / priority queue

## Insertion
O(log(P)) where P is the number of distinct prices at which we have bids for a new bid price / asks for a new ask price. This is the cost of insertion in the priority queue.
O(1) if there already exists an order at that limit price. (Insertion in a linked list)

## Deletion
To be implemented, but can be done in O(1) by lazily deleting the order from the order map.

## Execution
O(N) where N is the number of orders that are matched with the given order. Alternatively it can also be treated as O(Q) where Q is the quantity/size of the order since an order of size Q can be matched with at most Q orders.

## Modification
To be implemented, but can be done in O(1) by updating the order in the order map.

# Optimizations

## Use `u64` vs `float` for prices
We benefit from faster integer comparisons by using `u64` to denote the prices / ticks. We can use an additional precision field to convert decimal numbers to integers.

# Future Optimizations

## Parallel Request Handling
Since we're using DashMap for hashmaps, we can parallelize order updates / deletions from their actual execution.

# 🦀 rusty-orderbook
An attempt to build an orderbook exchange in Rust.

## 📋 Table of Contents

1. [Usage](#-usage)
2. [Testing](#-testing)
3. [Theoretical Analysis](#-theoretical-analysis)
   - [Best Bid and Best Ask](#best-bid-and-best-ask)
   - [Insertion](#insertion)
   - [Deletion](#deletion)
   - [Execution](#execution)
   - [Modification](#modification)
4. [Optimizations](#-optimizations)
   - [Use `u64` vs `float` for prices](#use-u64-vs-float-for-prices)
5. [Future Optimizations](#-future-optimizations)
   - [Parallel Request Handling](#parallel-request-handling)

## 🚀 Usage
1. Make sure you have installed the rust environment. (Further instructions at [rustup.rs](https://rustup.rs/))
2. To build the release / optimized binary: `cargo build --release`
3. Execute the binary: `./target/release/rusty-orderbook --input <INPUT_LOGS>` (You can try some sample logs in `sample_logs` folder

## 🧪 Testing
Run the `cargo test` command to run the unit tests.

## System Design
![Orderbook Order Flow](https://github.com/user-attachments/assets/6be300f4-5d0c-43e5-80bc-598925706738)

## 📊 Theoretical Analysis
The orderbook is implemented with the help of maps for mapping the order IDs, mapping order lists to the tick they are part of, and heaps for determining the best bid and best ask at any point in time in O(1).

### Best Bid and Best Ask
O(1) using heaps/priority queues.

### Insertion
- O(log(P)) where P is the number of distinct prices at which we have bids for a new bid price or asks for a new ask price. This is the cost of insertion in the priority queue.
- O(1) if there already exists an order at that limit price (insertion in a linked list).

### Deletion
Done in O(1) by lazily deleting the order from the order map.

### Execution
- O(N) where N is the number of orders that are matched with the given order.
- Alternatively, it can also be treated as O(Q), where Q is the quantity/size of the order, since an order of size Q can be matched with at most Q orders.

### Modification
Done in O(1) by updating the order in the order map.

## ⚙️ Optimizations

### Use `u64` vs `float` for prices
We benefit from faster integer comparisons by using `u64` to denote the prices/ticks. An additional precision field can be used to convert decimal numbers to integers.

## 🔮 Future Optimizations

### Parallel Request Handling
Since we're using DashMap for hashmaps, we can parallelize order updates/deletions from their actual execution.

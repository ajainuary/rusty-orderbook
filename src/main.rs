use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

mod errors;
mod orders;
mod requests;
mod orderbook;

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(author = "Anurag Jain <ajainuary@gmail.com>", version = "0.1.0", about = "An implementation of an orderbook exchange in Rust.", long_about = None)]
struct Args {
    /// Input file for order logs
    #[arg(short, long)]
    input: String
}
fn main() {
    let args = Args::parse();
    println!("Input file: {}", args.input);
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(args.input) {
        // Consumes the iterator, returns an (Optional) String
        let mut requests = Vec::new();
        for line in lines {
            if let Ok(log) = line {
                let request = requests::Request::try_from(log.as_ref());
                requests.push(request.unwrap());
            }
        }
        orderbook::process_requests(&mut requests);
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
use std::env;

use sunxdcc;

fn main() {
    let query = env::args().skip(1).next().unwrap();

    for result in sunxdcc::search(&query) {
        println!("{:?}", result);
    }
}

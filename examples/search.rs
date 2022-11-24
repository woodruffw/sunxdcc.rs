use std::env;



fn main() {
    let query = env::args().nth(1).unwrap();

    for result in sunxdcc::search(&query) {
        println!("{:?}", result);
    }
}

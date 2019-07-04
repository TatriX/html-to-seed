use std::io::{self, Read};
use html_to_seed::{convert, format};

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap();
    println!("{}", format(convert(buf)));
}

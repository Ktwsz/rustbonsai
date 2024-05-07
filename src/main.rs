pub mod bonsai;

use std::io;
use bonsai::{BonsaiTree};
pub mod app;
use app::{App,TERMINAL_BOUNDS};



fn main() ->io::Result<()> {
    let mut tree = BonsaiTree::new(TERMINAL_BOUNDS);
    tree.generate();
    tree.normalize();

    App::run(tree)
}

fn print_buffer(buffer: &Vec<Vec<char>>) {
    for y in (0..buffer[0].len()).rev() {
        for x in 0..buffer.len() {
            print!("{}", buffer[x][y]);
        }
        println!("");
    }
}



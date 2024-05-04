pub mod bonsai;
use bonsai::{BonsaiTree, AsciiChange};
use std::{thread, time};

const TERMINAL_BOUNDS: (u32, u32) = (100, 50);

fn main() {
    let mut tree = BonsaiTree::new(TERMINAL_BOUNDS);

    tree.generate();
    tree.normalize();

    let bounds: (usize, usize) = (TERMINAL_BOUNDS.0 as usize + 1, TERMINAL_BOUNDS.1 as usize + 1);
    let mut buffer = vec![vec![' '; bounds.1]; bounds.0];

    // tree.fill_buffer(&mut buffer);
    // print_buffer(&buffer);
    for _ in 0..100 {
        let ascii_changes = tree.animation_step();

        for change in ascii_changes {
            match change {
                AsciiChange::Change((x, y), c) => buffer[x][y] = c,
                AsciiChange::Stop => break,
                _ => (),
            }
        }

        print_buffer(&buffer);

        thread::sleep(time::Duration::from_millis(100));
    }
}

fn print_buffer(buffer: &Vec<Vec<char>>) {
    for y in 0..buffer[0].len() {
        for x in 0..buffer.len() {
            print!("{}", buffer[x][y]);
        }

        println!("");
    }
}

pub mod bonsai;

const TERMINAL_BOUNDS: (u32, u32) = (50, 50);
const FPS: u32 = 30;
const FRAMES_COUNT: u32 = 3;


fn main() {
    let mut tree = bonsai::BonsaiTree::new(TERMINAL_BOUNDS, FPS * FRAMES_COUNT);

    tree.generate();
    tree.normalize();

    let bounds: (usize, usize) = (TERMINAL_BOUNDS.0 as usize + 1, TERMINAL_BOUNDS.1 as usize + 1);
    let mut buffer = vec![vec![' '; bounds.1]; bounds.0];
    tree.fill_buffer(&mut buffer);

    for y in 0..bounds.1 {
        for x in 0..bounds.0  {
            print!("{}", buffer[x][y]);
        }

        println!("");
    }
}

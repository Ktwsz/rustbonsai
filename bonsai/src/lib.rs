mod bonsai_gen;

pub fn testing() {
    let mut tree = bonsai_gen::BonsaiTree::new();

    tree.generate();
    tree.normalize();

    let mut buffer = vec![vec![' '; 101]; 101];
    tree.fill_buffer(&mut buffer);

    for y in 0..101 {
        for x in 0..101 {
            print!("{}", buffer[x][y]);
        }

        println!("");
    }
}

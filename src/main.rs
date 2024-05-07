pub mod bonsai;

use std::io;

pub mod app;
use app::{App};

fn main() ->io::Result<()> {
    App::run()
}




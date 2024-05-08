pub mod bonsai;
use clap::Parser;
use std::io;
pub mod app;
use app::{App};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long,default_value_t=200, help = "Specify u64 number to generate seed for simulation if not specified will take from entropy")]
    seed: u64,
    #[arg(short, long, default_value_t = false, help= "If included will show live simulation")]
    live: bool,
}

fn main() ->io::Result<()> {
    let args = Args::parse();
    App::run(args.seed,args.live)
}




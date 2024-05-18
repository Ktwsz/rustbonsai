pub mod bonsai;
use clap::Parser;
use std::io;
pub mod app;
use app::App;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Specify u64 number to generate seed for simulation if not specified will be random")]
    seed: Option<u64>,
    #[arg(short, long, default_value_t = false, help = "If included will show live simulation")]
    live: bool,
    #[arg(short, long, default_value_t = 1, help = "Change color scheme: 1 Basic, 2 Cherry, 3 Maple, 4 Avatar")]
    theme: u16,
}

fn main() ->io::Result<()> {
    let args = Args::parse();
    App::run(args.seed,args.live, args.theme)
}




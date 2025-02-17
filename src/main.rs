use evaluator::evaluator::Evaluator;
use clap::Parser;
use tracing_subscriber;
use tracing::{Level, debug, warn, error, info};

#[derive(Parser)]
struct Args {
    #[arg(short, long="data-dir", help="The data directory to use")]
    data_dir: String,
    
    #[arg(short='v', long, action = clap::ArgAction::Count, help="Sets the verbose level. More v's more output")]
    verbose: u8
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logger = tracing_subscriber::fmt();

    let logger = match args.verbose {
        0 => logger.with_max_level(Level::INFO),
        1 => logger.with_max_level(Level::DEBUG),
        _ => logger.with_max_level(Level::TRACE),
    };

    logger.init();
    
    let mut eval = Evaluator::new(
        // "git+https://git.ole.blue/ole/nix-config",
        "path:///home/ole/nixos",
        // "hydraJobs"
        r#"nixosConfigurations."kartoffelkiste".config.system.build.toplevel"#
    );

    eval.start().await;
}

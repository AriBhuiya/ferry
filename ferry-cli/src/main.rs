use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "ferry", version, about, author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Serve(ServeArgs),
}

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Bind address (default: 127.0.0.1)
    #[arg(short = 'H', long = "host", default_value = "127.0.0.1")]
    pub host: String,

    /// Bind port (default: 3625 = DOCK on T9)
    #[arg(short = 'p', long = "port", default_value_t = 3625u16)]
    pub port: u16,

    /// Directory to save files (session root)
    #[arg(long = "dir", default_value_os = ".")]
    pub dir: PathBuf,

    // Following to be implemented later:
    // /// Require this pairing code to accept connections
    // #[arg(long = "code")]
    // pub code: Option<String>,
    //
    // /// Auto-approve incoming file lists
    // #[arg(long = "approve-all")]
    // pub approve_all: bool,
    //
    // /// Expose listings but reject writes
    // #[arg(long = "read-only", conflicts_with = "approve_all")]
    // pub read_only: bool,
    //
    // /// Refuse transfers larger than this
    // #[arg(long = "max-size")]
    // pub max_size: Option<String>,
    //
    // /// Confirm you understand public exposure when using --host 0.0.0.0 without --code
    // #[arg(long = "confirm-public")]
    // pub confirm_public: bool,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => {
            println!("Serving on {args:?}");
        }
    }
}



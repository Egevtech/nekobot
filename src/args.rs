use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum SCommand{
    /// Recreate database file from scratch
    #[command(short_flag='r')]
    RecreateDatabase{
        /// Recreate database file and exit
        #[arg(short='o', long)]
        recreate_only: bool,
    }
}

#[derive(Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    pub subcommand: SCommand,
}

impl Cmd {
    pub fn get_args() -> Self {
        Self::parse()
    }
}
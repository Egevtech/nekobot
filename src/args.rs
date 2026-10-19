use clap::Parser;

#[derive(Parser)]
pub struct Cmd {
    #[arg(short, long)]
    pub recreate_database: bool,
}

impl Cmd {
    pub fn get_args() -> Self {
        Self::parse()
    }
}
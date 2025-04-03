use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {}

#[derive(Parser, Debug)]
pub struct Args {
    /// The subcommand which should be executed
    #[command(subcommand)]
    pub command: Option<Command>,
}

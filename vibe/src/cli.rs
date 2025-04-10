use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// The output name to start hot reloading the config of the given output.
    pub output_name: Option<String>,
}

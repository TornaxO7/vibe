use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Show a list of input devices which you can choose from and set it in your config.
    #[arg(long)]
    pub show_input_devices: bool,

    /// The output name to start hot reloading the config of the given output.
    pub output_name: Option<String>,
}

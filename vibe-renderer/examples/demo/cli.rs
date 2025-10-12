use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ComponentName {
    Aurodio,

    BarsColorVariant,
    BarsFragmentCodeVariant,
    BarsPresenceGradientVariant,

    CircleCurvedVariant,

    FragmentCanvas,

    GraphColorVariant,
    GraphHorizontalGradientVariant,
    GraphVerticalGradientVariant,

    RadialColorVariant,
    RadialHeightGradientVariant,

    ChessyBoxVariant,

    TextureValueNoise,
    TextureSdf,

    WallpaperPulseEdges,
}

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Choose which component should be displayed in the demo.
    #[arg(value_enum, long, short)]
    pub component_name: Option<ComponentName>,

    /// Show a list of all available output devices which this demo should use.
    #[arg(long)]
    pub show_output_devices: bool,

    /// Set the name of the output device for the demo.
    #[arg(long)]
    pub output_device_name: Option<String>,
}

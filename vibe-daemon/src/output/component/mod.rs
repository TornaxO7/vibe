use bars::BarsConfig;
use fragment_canvas::FragmentCanvasConfig;

pub mod bars;
pub mod fragment_canvas;

#[derive(Debug)]
pub enum ComponentConfig {
    Bars(BarsConfig),
    FragmentCanvas(FragmentCanvasConfig),
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars(BarsConfig::default())
    }
}

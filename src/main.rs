mod model;
mod tui;
mod update;
mod utils;

use color_eyre::Result;
use tui::run;

fn main() -> Result<()> {
    run()
}

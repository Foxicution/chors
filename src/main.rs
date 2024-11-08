mod model;
mod tui;
mod update;
mod utils;

use color_eyre::Result;
use tui::run::run;

fn main() -> Result<()> {
    run()
}

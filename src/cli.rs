use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("Chors - Task Manager.")
        .version("1.0")
        .about("A simple, yet powerful task manager in the terminal.")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Sets a custom file for persistence"),
        )
}

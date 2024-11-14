use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("Chors - Task Manager")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A simple, yet powerful task manager for the terminal.")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help(
                    "Gives the location of the configuration file. \
                    If the file doesn't exist, it is created instead.",
                ),
        )
}

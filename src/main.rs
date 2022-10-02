use log::{error, warn};
use std::path::Path;
use std::fs;

use calendar_lib::cli::{self, Cli, Commands};



fn main() {
    // Initialize logging
    env_logger::init();

    let mut data_dir = std::env::current_dir()
        .expect("Cannot access the current directory");
    data_dir.push("data");
    if let Err(e) = fs::create_dir_all(data_dir.as_path()) {
        error!("Data directory creation failed: {e}");
        return;
    }

    let args = Cli::parse_cli();

    let (readonly, res) = cli::Cli::exec_commands(&args, &data_dir.as_path());
    
    let mut cal = res.expect("Error opening the calendar")
        .expect("Missing calendar");
    let result = match (args.subcommand, readonly) {
        (Some(Commands::Add(x)), false) => match cli::handle_add(&mut cal, x) {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                false
            }
        },
        (Some(Commands::Remove(rm)), false) => cli::handle_remove(&mut cal, rm),
        (Some(Commands::List(l)), _) => cli::handle_list(&cal, l),
        (Some(Commands::Set(params)), false) => cli::handle_params(&mut cal, params),
        (Some(_), true) => {
            warn!(
                "Calendar {} cannot be modified! (rerun with --edit)",
                cal.get_name()
            );
            eprintln!(
				"Calendar {} cannot be modified! (rerun with --edit)",
                cal.get_name()
            );
            false
        }
        (None, _) => true, // no commands to perform => ok to save result
    };

    if result
        && !cli::save_calendar(
            &cal,
            &data_dir.join(Path::new(cal.get_name()).with_extension("json")),
        )
    {
        warn!("Cannot write calendar {} to {}", cal, data_dir.display());
        eprintln!("Cannot write calendar {} to {}", cal, data_dir.display());
    }
}

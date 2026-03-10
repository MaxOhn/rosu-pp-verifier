use clap::ArgAction;
use rosu_pp_verifier::commands;

fn main() {
    #[derive(clap::Parser)]
    enum Args {
        /// Use osu!lazer to calculate attributes for all maps and store them
        Calculate {
            /// Overwrite the output file if it exists already
            #[arg(long, action = ArgAction::Set)]
            overwrite: bool,
        },
        /// Read stored attributes and compare them with rosu-pp's attributes
        Compare,
        /// Read stored attributes and recalculate some of them
        Recalculate,
        /// Load scores from a data dump and store them
        Load {
            /// Specify a gamemode. Otherwise, all modes will be loaded
            #[arg(long)]
            mode: Option<u8>,
        },
        /// Read loaded scores and compare their osu-tools & rosu-pp attributes
        Loaded {
            /// Amount of minutes scores will be compared. Uncapped by default.
            #[arg(long)]
            minutes: Option<u64>,
        },
    }

    dotenvy::dotenv().expect("Failed to load .env file");

    match <Args as clap::Parser>::parse() {
        Args::Calculate { overwrite } => commands::calculate(overwrite),
        Args::Compare => commands::compare(),
        Args::Recalculate => commands::recalculate(),
        Args::Load { mode } => commands::load(mode),
        Args::Loaded { minutes } => commands::loaded(minutes),
    }
}

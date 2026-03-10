# rosu-pp-verifier

Binary with various subcommands to compare the difficulty & performance calculation outputs of lazer with rosu-pp.

# How to use

## Use scores from https://data.ppy.sh/

1. Download scores for a mode, e.g. `2025_11_01_performance_mania_random_10000.tar.bz2`
2. Extract the file. Only the `scores.sql` file is needed.
3. Ensure you have all required beatmaps. If you are unsure, download and extract the corresponding `osu_files.tar.bz2` file.
4. Have a local clone of the [osu](https://github.com/ppy/osu) and [osu-tools](https://github.com/ppy/osu-tools) repositories.
5. Checkout the relevant commit in both repositories and ensure that osu-tools uses the local osu repository.
6. Compile osu-tools' `PerformanceCalculator`.
7. Rename the `.env.example` file to `.env` and fill in the required environment variables.
8. Run `cargo run --release -- load [--mode <MODE>]` to load prepared scores from the sql file(s).
9. Run `cargo run -- loaded [--minutes <MINUTES>]` to compare the loaded scores with rosu-pp.

## Simulate scores with osu-tools and compare with rosu-pp

TODO: write readme

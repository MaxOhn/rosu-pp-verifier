use std::{env, sync::LazyLock};

/// Ensuring assertions at compile time
macro_rules! const_assert {
    ( $x:expr $(,)? ) => {
        #[allow(unknown_lints, clippy::eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

pub mod commands;
pub mod model;
mod util;

static MAP_PATH: LazyLock<String> = LazyLock::new(|| env::var("MAP_PATH").unwrap());
static PERF_CALC: LazyLock<String> = LazyLock::new(|| env::var("PERF_CALC").unwrap());
static SIMULATE_OUTPUT: LazyLock<String> = LazyLock::new(|| env::var("SIMULATE_OUTPUT").unwrap());
static LOAD_OUTPUT: LazyLock<String> = LazyLock::new(|| env::var("LOAD_OUTPUT").unwrap());

/// Separator between serialized rkyv entries
const SEPARATOR: &[u8] = b"#END#";

/// Additional alignment to account for separator within serialized content
const MAX_ALIGN: usize = 8;

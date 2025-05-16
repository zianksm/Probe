use std::{ffi::OsString, os::unix::ffi::OsStrExt};

use clap::Parser;
use forge::{
    opts::Forge,
    result::{TestOutcome, TestResult},
};
use foundry_cli::utils::LoadConfig;
use foundry_config::Config;
use foundry_debugger::Debugger;

struct DebugAdapter;

impl DebugAdapter {
    pub fn collect_trace_and_config(test: &str) -> (TestResult, Config ) {
        let cmd = format!("forge test --mt {test} --decode-internal");
        let cmd = cmd.split_whitespace().map(OsString::from);

        match Forge::parse_from(cmd).cmd {
            forge::opts::ForgeSubcommand::Test(test_args) => {

                let config = test_args.load_config().expect("should be able to load config");

                let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
                let outcome = rt.block_on(async move {
                    test_args.run().await.expect("test should run successfully")
                });

                let (_, _, result) = outcome.remove_first().expect("no tests were executed");

                (result, config)
            }
            _ => unreachable!(),
        }
    }

    pub fn debug(test: &str) {
        Debugger
        let (traces, config) = Self::collect_trace_and_config(test);
            // get library by constructing a temporary test runner and cloning the library
    }
}

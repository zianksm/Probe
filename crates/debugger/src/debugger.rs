use std::{
    collections::BTreeSet, default, ffi::OsString, os::unix::ffi::OsStrExt, path::PathBuf,
    sync::Arc,
};

use clap::Parser;
use forge::{
    opts::Forge,
    result::{TestOutcome, TestResult},
    revm::primitives::{map::AddressHashMap, Bytes},
    traces::{debug::ContractSources, CallTraceArena, Traces},
};
use foundry_cli::utils::LoadConfig;
use foundry_common::{compile::ProjectCompiler, evm::Breakpoints, get_contract_name};
use foundry_compilers::{artifacts::Libraries, solc::SolcCompiler};
use foundry_config::Config;
use foundry_debugger::DebugNode;
use foundry_evm_traces::CallTraceDecoder;
use revm_inspectors::tracing::types::TraceMemberOrder;

pub struct Debugger {
    ctx: Context,
}

impl Debugger {
    pub fn new(
        debug_arena: Vec<DebugNode>,
        identified_contracts: AddressHashMap<String>,
        contracts_sources: ContractSources,
        breakpoints: Breakpoints,
    ) -> Self {
        Self {
            ctx: Context::new(
                debug_arena,
                identified_contracts,
                contracts_sources,
                breakpoints,
            ),
        }
    }
}

struct Context {
    pub debug_arena: Vec<DebugNode>,
    pub identified_contracts: AddressHashMap<String>,
    /// Source map of contract sources
    pub contracts_sources: ContractSources,
    pub breakpoints: Breakpoints,
}

impl Context {
    fn new(
        debug_arena: Vec<DebugNode>,
        identified_contracts: AddressHashMap<String>,
        contracts_sources: ContractSources,
        breakpoints: Breakpoints,
    ) -> Self {
        Self {
            debug_arena,
            identified_contracts,
            contracts_sources,
            breakpoints,
        }
    }
}
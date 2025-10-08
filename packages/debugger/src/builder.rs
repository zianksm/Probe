use std::{
  collections::BTreeSet, default, ffi::OsString, os::unix::ffi::OsStrExt, path::PathBuf, sync::Arc,
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

use crate::engine::Debugger;

pub struct DebuggerBuilder;

pub struct TestResultAndTraces {
  result: TestResult,
  config: Config,
  sources: BTreeSet<PathBuf>,
  trace_decoder: CallTraceDecoder,
}

impl DebuggerBuilder {
  fn collect_project_infos(test: &str) -> TestResultAndTraces {
    let cmd = format!("forge test --mt {test} --decode-internal");
    let cmd = cmd.split_whitespace().map(OsString::from);

    match Forge::parse_from(cmd).cmd {
      forge::opts::ForgeSubcommand::Test(test_args) => {
        let config = test_args
          .load_config()
          .expect("should be able to load config");

        let filter = test_args.filter(&config).unwrap();

        let sources = test_args.get_sources_to_compile(&config, &filter).unwrap();

        let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
        let mut outcome =
          rt.block_on(async move { test_args.run().await.expect("test should run successfully") });

        let (_, _, result) = outcome.remove_first().expect("no tests were executed");

        let trace_decoder = outcome.last_run_decoder.unwrap();

        TestResultAndTraces {
          result,
          config,
          sources,
          trace_decoder,
        }
      }
      _ => unreachable!(),
    }
  }

  fn fetch_complete_sources(config: &Config, sources: BTreeSet<PathBuf>) -> ContractSources {
    let compiler = ProjectCompiler::new()
      .dynamic_test_linking(true)
      .quiet(true)
      .files(sources);

    // the test should be already compiled since we already run the test to collect the traces
    let project = config.project().unwrap();
    let root = &project.paths.root;

    let output = compiler.compile(&project).unwrap();

    let dummy_runner = forge::MultiContractRunnerBuilder::new(Arc::new(config.to_owned()))
      .build::<SolcCompiler>(root, &output, Default::default(), Default::default())
      .unwrap();

    let libs = dummy_runner.libraries;

    ContractSources::from_project_output(&output, root, Some(&libs)).unwrap()
  }

  pub fn build(test: &str) -> Debugger {
    // Debugger
    let TestResultAndTraces {
      result,
      config,
      sources,
      trace_decoder,
    } = Self::collect_project_infos(test);

    let sources = Self::fetch_complete_sources(&config, sources.to_owned());
    let traces = result
      .traces
      .iter()
      .filter(|(t, _)| t.is_execution())
      .cloned()
      .collect::<Traces>();

    let debug_arena = Self::into_nodes(traces);

    let identified_contracts = Self::get_identified_contracts_from_decoder(trace_decoder);

    let breakpoints = result.breakpoints;

    Debugger::new(debug_arena, identified_contracts, sources, breakpoints)

  }

  fn get_identified_contracts_from_decoder(decoder: CallTraceDecoder) -> AddressHashMap<String> {
    let mut out = <AddressHashMap<String> as Default>::default();

    let iter = decoder
      .contracts
      .iter()
      .map(|(k, v)| (*k, get_contract_name(v).to_string()));

    Extend::extend(&mut out, iter);

    out
  }

  fn into_nodes(traces: Traces) -> Vec<DebugNode> {
    let mut out = Vec::new();

    for (_, sparse_arena) in traces {
      let arena = sparse_arena.arena;

      #[derive(Debug, Clone, Copy)]
      struct PendingNode {
        node_idx: usize,
        steps_count: usize,
      }

      fn inner(arena: &CallTraceArena, node_idx: usize, out: &mut Vec<PendingNode>) {
        let mut pending = PendingNode {
          node_idx,
          steps_count: 0,
        };
        let node = &arena.nodes()[node_idx];
        for order in &node.ordering {
          match order {
            TraceMemberOrder::Call(idx) => {
              out.push(pending);
              pending.steps_count = 0;
              inner(arena, node.children[*idx], out);
            }
            TraceMemberOrder::Step(_) => {
              pending.steps_count += 1;
            }
            _ => {}
          }
        }
        out.push(pending);
      }
      let mut nodes = Vec::new();
      inner(&arena, 0, &mut nodes);

      let mut arena_nodes = arena.into_nodes();

      for pending in nodes {
        let steps = {
          let other_steps = arena_nodes[pending.node_idx]
            .trace
            .steps
            .split_off(pending.steps_count);
          std::mem::replace(&mut arena_nodes[pending.node_idx].trace.steps, other_steps)
        };

        // Skip nodes with empty steps as there's nothing to display for them.
        if steps.is_empty() {
          continue;
        }

        let call = &arena_nodes[pending.node_idx].trace;
        let calldata = if call.kind.is_any_create() {
          Bytes::new()
        } else {
          call.data.clone()
        };
        let node = DebugNode::new(call.address, call.kind, steps, calldata);

        out.push(node);
      }
    }

    out
  }
}

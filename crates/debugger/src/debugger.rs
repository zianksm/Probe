use std::{
    collections::BTreeSet, default, ffi::OsString, os::unix::ffi::OsStrExt, path::PathBuf,
    sync::Arc,
};

use clap::Parser;
use forge::{
    opts::Forge,
    result::{TestOutcome, TestResult},
    revm::{
        interpreter::OpCode,
        primitives::{map::AddressHashMap, Bytes},
    },
    traces::{debug::ContractSources, CallTraceArena, Traces},
};
use foundry_cli::utils::LoadConfig;
use foundry_common::{compile::ProjectCompiler, evm::Breakpoints, get_contract_name};
use foundry_compilers::{artifacts::Libraries, solc::SolcCompiler};
use foundry_config::Config;
use foundry_debugger::DebugNode;
use foundry_evm_traces::CallTraceDecoder;
use revm_inspectors::tracing::types::{CallTraceStep, TraceMemberOrder};

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

    fn current_call_ctx(&self) -> &DebugNode {
        &self.ctx.debug_arena[self.ctx.call_index]
    }

    fn prev_call_ctx(&self) -> &DebugNode {
        &self.ctx.debug_arena[self.ctx.call_index - 1]
    }

    fn is_jmp(step: &CallTraceStep, prev: &CallTraceStep) -> bool {
        match matches!(
            prev.op,
            OpCode::JUMP
                | OpCode::JUMPI
                | OpCode::JUMPF
                | OpCode::RJUMP
                | OpCode::RJUMPI
                | OpCode::RJUMPV
                | OpCode::CALLF
                | OpCode::RETF
        ) {
            true => true,
            false => {
                // for instructions that has data associated with it, PUSH20 PUSH32 etc since it's embedded in the PC, we must consider for it also here
                let immediate_len = prev.immediate_bytes.as_ref().map_or(0, |b| b.len());

                if step.pc != prev.pc + 1 + immediate_len {
                    true
                } else {
                    step.code_section_idx != prev.code_section_idx
                }
            }
        }
    }

    fn remaining_steps(&self) -> &[CallTraceStep] {
        &self.current_call_ctx().steps[self.ctx.current_step..]
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Step => self.step(),
            Action::StepBack => self.step_back(),
            Action::StepInto => todo!(),
            Action::StepOut => todo!(),
            Action::Continue => todo!(),
            Action::Stop => todo!(),
        }
    }

    /// default into step-over, skip calls unless an explicit step into is called
    fn step(&mut self) {
        let current_depth = self.ctx.call_depth;

        loop {
            if self.ctx.call_index >= self.ctx.debug_arena.len() - 1 {
                break;
            }

            self.handle_call_depth(false);

            if self.ctx.current_step < self.total_steps() {
                self.ctx.current_step += 1;
            } else {
                self.ctx.current_step = 0;
                self.ctx.call_index += 1;
            }

            if current_depth >= self.ctx.call_depth {
                break;
            }
        }
    }

    // doesn't support EOF opcode yet
    fn handle_call_depth(&mut self, step_back: bool) {
        let op = self.current_call_ctx().steps[self.ctx.current_step].op;

        if matches!(
            op,
            OpCode::CALL
                | OpCode::DELEGATECALL
                | OpCode::STATICCALL
                | OpCode::CREATE
                | OpCode::CREATE2
        ) {
            if step_back {
                self.ctx.call_depth -= 1
            } else {
                self.ctx.call_depth += 1
            };
        }

        if matches!(
            op,
            OpCode::RETURN | OpCode::REVERT | OpCode::STOP | OpCode::SELFDESTRUCT | OpCode::INVALID
        ) {
            if step_back {
                self.ctx.call_depth += 1
            } else {
                self.ctx.call_depth -= 1
            };
        }
    }

    fn step_back(&mut self) {
        let current_depth = self.ctx.call_depth;

        loop {
            self.handle_call_depth(true);

            if self.ctx.current_step > 0 {
                self.ctx.current_step -= 1;
            } else {
                self.ctx.call_index -= 1;
                self.ctx.current_step = self.current_call_ctx().steps.len() - 1;
            }

            if current_depth <= self.ctx.call_depth {
                break;
            }
        }
    }

    fn total_steps(&self) -> usize {
        self.current_call_ctx().steps.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Step,
    StepBack,
    StepInto,
    StepOut,
    Continue,
    Stop,
}

struct Context {
    pub debug_arena: Vec<DebugNode>,
    pub identified_contracts: AddressHashMap<String>,
    /// Source map of contract sources
    pub contracts_sources: ContractSources,
    pub breakpoints: Breakpoints,
    pub current_step: usize,
    pub call_index: usize,
    pub opcode_list: Vec<String>,
    pub call_depth: usize,
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
            call_depth: 0,
            current_step: 0,
            call_index: 0,
            opcode_list: vec![],
        }
    }
}

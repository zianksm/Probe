#![allow(static_mut_refs)]
use debugger::Debugger;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{any::Any, cell::RefCell, sync::atomic};

pub mod builder;
pub mod core;

static mut INSTANCE: Option<Debugger> = Option::None;

/// the access is guaranteed to be single threaded since we will package this into a
/// js addon via napi
fn with_instance<F, R>(f: F) -> R
where
  F: FnOnce(&mut Debugger) -> R,
  R: Any,
{
  unsafe {
    let mut instance = INSTANCE.take().expect("no debugger instance found");

    let r = f(&mut instance);

    INSTANCE.replace(instance);

    r
  }
}

#[napi]
fn init(path: String) {
  let debugger = builder::DebuggerBuilder::build(&path);

  unsafe {
    INSTANCE.replace(debugger);
  }
}

// debugging core functions
#[napi]
fn step() {
  with_instance(|d| d.handle_action(debugger::Action::Step));
}

#[napi]
fn step_back() {
  with_instance(|d| d.handle_action(debugger::Action::StepBack));
}

#[napi]
fn opcode_list() -> Vec<String> {
  with_instance(|d| d.opcode_list())
}

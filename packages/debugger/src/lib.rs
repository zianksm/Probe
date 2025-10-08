#![allow(static_mut_refs)]
use engine::Debugger;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{
  any::Any,
  cell::RefCell,
  panic::{catch_unwind, PanicHookInfo, PanicInfo},
  sync::atomic,
};

use crate::types::FlatDebugNode;

pub mod builder;
pub mod engine;
pub mod types;

static mut INSTANCE: Option<Debugger> = Option::None;
// THIS NEEDS TO BE RE-ASSIGNED ON EVERY ADDON CALL
static mut ENV_REF: Option<Env> = Option::None;

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

fn set_hook(env: Env) {
  unsafe {
    ENV_REF.replace(env);

    std::panic::set_hook(Box::new(|info| {
      let env_extern = ENV_REF.take().unwrap();

      if let Some(s) = info.payload().downcast_ref::<&str>() {
        env_extern.throw_error(*s, None);
      } else if let Some(s) = info.payload().downcast_ref::<String>() {
        env_extern.throw_error(s.as_str(), None);
      } else {
        env_extern.throw_error("anonymous panic", None);
      }
      
      ENV_REF.replace(env_extern)
    }));
  }
}

#[napi]
fn init(env: Env, path: String) {
  set_hook(env);

  let debugger = builder::DebuggerBuilder::build(&path);

  unsafe {
    INSTANCE.replace(debugger);
  }
}

// debugging core functions
#[napi]
fn step() {
  with_instance(|d| d.handle_action(engine::Action::Step));
}

#[napi]
fn step_back() {
  with_instance(|d| d.handle_action(engine::Action::StepBack));
}

#[napi]
fn opcode_list() -> Vec<String> {
  with_instance(|d| d.opcode_list())
}

#[napi]
fn call_ctx() -> FlatDebugNode {
  with_instance(|d| d.current_call_ctx().clone().into())
}

#[napi]
fn test(env: Env) -> String {
  set_hook(env);

   env.throw_error("anonymous panic", None);
  let result = catch_unwind(|| panic!("test panic"));

  match result {
    Ok(_) => "Test".to_string(),
    Err(cause) => {
      // Try &str first (most common)
      if let Some(s) = cause.downcast_ref::<&str>() {
        s.to_string()
      }
      // Then try String
      else if let Some(s) = cause.downcast_ref::<String>() {
        s.clone()
      }
      // Fallback
      else {
        "Unknown panic".to_string()
      }
    }
  }
}

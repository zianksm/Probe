#![allow(static_mut_refs)]
use debugger::Debugger;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{cell::RefCell, sync::atomic};

pub mod builder;
pub mod debugger;

static mut INSTANCE: Option<Debugger> = Option::None;

/// the access is guaranteed to be single threaded since we will package this into a
/// js addon via napi
fn with_instance<F>(f: F)
where
    F: FnOnce(&mut Debugger),
{
    unsafe {
        let mut instance = INSTANCE.take().expect("no debugger instance found");

        f(&mut instance);

        INSTANCE.replace(instance);
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

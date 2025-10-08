use foundry_debugger::DebugNode;
use napi_derive::napi;
use revm_inspectors::tracing::types::{
  CallTraceStep, DecodedInternalCall, DecodedTraceStep, StorageChange, TraceMemberOrder,
};

/// Flattened version of DebugNode for napi compatibility
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct FlatDebugNode {
  /// Execution context address as hex string
  pub address: String,
  /// The kind of call as string representation
  pub kind: String,
  /// Calldata as hex string
  pub calldata: String,
  /// The debug steps (flattened)
  pub steps: Vec<FlatCallTraceStep>,
}

impl From<DebugNode> for FlatDebugNode {
  fn from(value: DebugNode) -> Self {
    let steps = value
      .steps
      .into_iter()
      .map(|step| step.into())
      .collect::<Vec<_>>();

    Self {
      address: value.address.to_string(),
      kind: value.kind.to_string(),
      calldata: value.kind.to_string(),
      steps,
    }
  }
}

/// Flattened version of CallTraceStep for napi compatibility
#[napi(object)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatCallTraceStep {
  /// Call depth
  pub depth: i64,
  /// Program counter before step execution
  pub pc: i64,
  /// Opcode as string (u8 value)
  pub op: String,
  /// Current contract address as hex string
  pub contract: String,
  /// Stack before step execution as hex strings
  pub stack: Option<Vec<String>>,
  /// The new stack items placed by this step as hex strings
  pub push_stack: Option<Vec<String>>,
  /// Memory before step execution as hex string
  pub memory: Option<String>,
  /// Returndata before step execution as hex string
  pub returndata: String,
  /// Remaining gas before step execution
  pub gas_remaining: i64,
  /// Gas refund counter before step execution
  pub gas_refund_counter: i64,
  /// Total gas used before step execution
  pub gas_used: i64,
  /// Gas cost of step execution
  pub gas_cost: i64,
  /// Change of the contract state after step execution (flattened)
  pub storage_change: Option<FlatStorageChange>,
  /// Final status of the step as string
  pub status: Option<u8>,
  /// Immediate bytes of the step as hex string
  pub immediate_bytes: Option<String>,
  /// Optional complementary decoded step data (flattened)
  pub decoded: Option<FlatDecodedTraceStep>,
}

impl From<CallTraceStep> for FlatCallTraceStep {
  fn from(value: CallTraceStep) -> Self {
    let stack = match value.stack {
      Some(items) => {
        let mut stringified = vec![];

        for item in items {
          stringified.push(item.to_string());
        }

        Some(stringified)
      }

      None => None,
    };

    let push_stack = match value.push_stack {
      Some(items) => {
        let mut stringified = vec![];

        for item in items {
          stringified.push(item.to_string());
        }

        Some(stringified)
      }

      None => None,
    };

    let memory = match value.memory {
      Some(mems) => Some(mems.as_bytes().to_string()),
      None => None,
    };

    Self {
      depth: value.depth as i64,
      pc: value.pc as i64,
      op: value.op.as_str().to_string(),
      contract: value.contract.to_string(),
      stack: stack,
      push_stack: push_stack,
      memory: memory,
      returndata: value.returndata.to_string(),
      gas_remaining: value.gas_remaining as i64,
      gas_refund_counter: value.gas_refund_counter as i64,
      gas_used: value.gas_used as i64,
      gas_cost: value.gas_cost as i64,
      storage_change: value.storage_change.and_then(|diff| Some(diff.into())),
      status: value.status.and_then(|status| Some(status as u8)),
      immediate_bytes: value
        .immediate_bytes
        .and_then(|bytes| Some(bytes.to_string())),
      decoded: value.decoded.and_then(|decoded| Some(decoded.into())),
    }
  }
}

/// Flattened version of StorageChange for napi compatibility
#[napi(object)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatStorageChange {
  /// Key of the storage slot as hex string
  pub key: String,
  /// Current value of the storage slot as hex string
  pub value: String,
  /// The previous value of the storage slot as hex string
  pub had_value: Option<String>,
  /// How this storage was accessed as string
  pub reason: String,
}

impl From<StorageChange> for FlatStorageChange {
  fn from(value: StorageChange) -> Self {
    let reason = match value.reason {
      revm_inspectors::tracing::types::StorageChangeReason::SLOAD => "SLOAD".to_string(),
      revm_inspectors::tracing::types::StorageChangeReason::SSTORE => "SSTORE".to_string(),
    };

    Self {
      key: value.key.to_string(),
      value: value.value.to_string(),
      had_value: value.had_value.and_then(|v| Some(v.to_string())),
      reason: reason,
    }
  }
}

/// Flattened version of DecodedTraceStep for napi compatibility
// #[napi_derive::napi(object)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[napi]
pub enum FlatDecodedTraceStep {
  /// Decoded internal function call with end index
  InternalCall {
    decoded_internal_call: FlatDecodedInternalCall,
    call_idx: i64,
  },
  /// Arbitrary line representing the step
  Line(String),
}

impl From<DecodedTraceStep> for FlatDecodedTraceStep {
  fn from(value: DecodedTraceStep) -> Self {
    match value {
      DecodedTraceStep::InternalCall(decoded_internal_call, call_idx) => Self::InternalCall {
        decoded_internal_call: decoded_internal_call.into(),
        call_idx: call_idx as i64,
      },
      DecodedTraceStep::Line(line) => Self::Line(line),
    }
  }
}

/// Flattened version of DecodedInternalCall for napi compatibility
#[napi(object)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatDecodedInternalCall {
  /// Name of the internal function
  pub func_name: String,
  /// Input arguments of the internal function
  pub args: Option<Vec<String>>,
  /// Optional decoded return data
  pub return_data: Option<Vec<String>>,
}

impl From<DecodedInternalCall> for FlatDecodedInternalCall {
  fn from(value: DecodedInternalCall) -> Self {
    Self {
      func_name: value.func_name,
      args: value.args,
      return_data: value.return_data,
    }
  }
}

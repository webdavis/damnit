use crate::config::HookEvent;
use crate::errors::Refusal;
use crate::ports::{FunctionFailure, FunctionFault, HookFailure};

pub(super) fn hook_samples() -> Vec<Refusal> {
    vec![
        Refusal::HookRefused(HookFailure {
            event: HookEvent::TaskDone,
            failure: FunctionFailure {
                file: "config.lua".into(),
                line: Some(3),
                fault: FunctionFault::RanOver,
            },
        }),
        Refusal::StartedByHook("task_new".into()),
    ]
}

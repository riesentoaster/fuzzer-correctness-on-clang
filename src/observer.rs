use std::borrow::Cow;

use libafl::observers::Observer;
use libafl_bolts::Named;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CorrectnessObserver {
    #[serde(skip)]
    step_ptr: *mut usize,
    step: usize,
    #[serde(skip)]
    name: std::borrow::Cow<'static, str>,
}

impl CorrectnessObserver {
    pub fn new(step_ptr: &mut [u8], name: String) -> Self {
        Self {
            step_ptr: step_ptr.as_mut_ptr().cast(),
            step: 0,
            name: Cow::Owned(name),
        }
    }

    pub fn step(&self) -> usize {
        self.step
    }
}

impl Named for CorrectnessObserver {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}

impl<I, S> Observer<I, S> for CorrectnessObserver {
    fn pre_exec(&mut self, _state: &mut S, _input: &I) -> Result<(), libafl::Error> {
        self.step = 0;
        unsafe { *self.step_ptr = 0 };
        Ok(())
    }

    fn post_exec(
        &mut self,
        _state: &mut S,
        _input: &I,
        _exit_kind: &libafl::executors::ExitKind,
    ) -> Result<(), libafl::Error> {
        self.step = unsafe { *self.step_ptr };
        Ok(())
    }
}

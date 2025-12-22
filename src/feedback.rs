use std::{borrow::Cow, marker::PhantomData};

use libafl::{
    events::{Event, EventFirer, EventWithStats},
    feedbacks::{Feedback, StateInitializer},
    monitors::stats::{AggregatorOps, UserStats, UserStatsValue},
    state::HasExecutions,
    Error, HasNamedMetadata,
};
use libafl_bolts::{
    tuples::{Handle, Handled as _, MatchName, MatchNameRef as _},
    Named, SerdeAny,
};
use serde::{Deserialize, Serialize};

use crate::observer::CorrectnessObserver;

pub struct ReportCorrectnessFeedback {
    observer: Handle<CorrectnessObserver>,
}

impl ReportCorrectnessFeedback {
    pub fn new(observer: &CorrectnessObserver) -> Self {
        Self {
            observer: observer.handle(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SerdeAny)]
struct CorrectnessMetadata {
    counts: Vec<usize>,
}

static MAX_CORRECTNESS_STEPS: usize = 1 << 10;

impl CorrectnessMetadata {
    pub fn new() -> Self {
        Self {
            counts: vec![0; MAX_CORRECTNESS_STEPS + 1],
        }
    }
}

impl<EM, I, OT, S> Feedback<EM, I, OT, S> for ReportCorrectnessFeedback
where
    EM: EventFirer<I, S>,
    S: HasExecutions + HasNamedMetadata,
    OT: MatchName,
{
    fn append_metadata(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        observers: &OT,
        _testcase: &mut libafl::corpus::Testcase<I>,
    ) -> Result<(), libafl::Error> {
        let observer = observers.get(&self.observer).ok_or_else(|| {
            Error::illegal_state(format!("Observer {} not found", self.observer.name()))
        })?;
        let metadata = state.named_metadata_or_insert_with(self.name(), CorrectnessMetadata::new);
        let step = observer.step();
        if step == 0 || step > MAX_CORRECTNESS_STEPS {
            // Skip if step is invalid
            // return Err(Error::illegal_state(format!(
            //     "Step {} is out of bounds",
            //     step
            // )));
        }
        metadata.counts[step] += 1;
        let total_hits = metadata.counts.iter().sum::<usize>();
        let stringified = metadata
            .counts
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(i, c)| format!("{}: {:.3}", i, *c as f64 / total_hits as f64))
            .collect::<Vec<String>>()
            .join(", ");
        manager.fire(
            state,
            EventWithStats::with_current_time(
                Event::UpdateUserStats {
                    name: self.name().clone(),
                    value: UserStats::new(
                        UserStatsValue::String(Cow::Owned(stringified)),
                        AggregatorOps::Avg,
                    ),
                    phantom: PhantomData,
                },
                *state.executions(),
            ),
        )?;
        Ok(())
    }
}

impl Named for ReportCorrectnessFeedback {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("correctness")
    }
}

impl<S> StateInitializer<S> for ReportCorrectnessFeedback
where
    S: HasNamedMetadata,
{
    fn init_state(&mut self, state: &mut S) -> Result<(), Error> {
        state.add_named_metadata_checked(self.name(), CorrectnessMetadata::new())
    }
}

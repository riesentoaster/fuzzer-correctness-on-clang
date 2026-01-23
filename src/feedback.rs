use std::{borrow::Cow, collections::BTreeMap, marker::PhantomData};

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
    counts: BTreeMap<usize, usize>,
    inter_report_count: usize,
}

impl CorrectnessMetadata {
    pub fn new() -> Self {
        Self {
            counts: BTreeMap::new(),
            inter_report_count: 0,
        }
    }
}

impl<EM, I, OT, S> Feedback<EM, I, OT, S> for ReportCorrectnessFeedback
where
    EM: EventFirer<I, S>,
    S: HasExecutions + HasNamedMetadata,
    OT: MatchName,
{
    fn is_interesting(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        _input: &I,
        observers: &OT,
        _exit_kind: &libafl::executors::ExitKind,
    ) -> Result<bool, Error> {
        let observer = observers.get(&self.observer).ok_or_else(|| {
            Error::illegal_state(format!("Observer {} not found", self.observer.name()))
        })?;
        let metadata = state.named_metadata_or_insert_with(self.name(), CorrectnessMetadata::new);
        let failure_step = observer.step();

        metadata
            .counts
            .entry(failure_step)
            .and_modify(|f| *f += 1)
            .or_insert(1);

        metadata.inter_report_count += 1;

        let report_relative = metadata.inter_report_count % 100 == 0;
        let report_absolute = (metadata.inter_report_count + 50) % 100 == 0;

        let total_hits = if report_relative {
            metadata.counts.values().sum::<usize>()
        } else {
            0
        };

        let stringified_relative = metadata
            .counts
            .iter()
            .map(|(k, v)| format!("{}: {:.3}", k, *v as f64 / total_hits as f64))
            .intersperse(", ".to_string())
            .collect::<String>();
        let stringified_absolute = metadata
            .counts
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .intersperse(", ".to_string())
            .collect::<String>();
        if report_relative {
            manager.fire(
                state,
                EventWithStats::with_current_time(
                    Event::UpdateUserStats {
                        name: Cow::Owned(format!("{}-relative", self.name())),
                        value: UserStats::new(
                            UserStatsValue::String(Cow::Owned(stringified_relative)),
                            AggregatorOps::Avg,
                        ),
                        phantom: PhantomData,
                    },
                    *state.executions(),
                ),
            )?;
        }
        if report_absolute {
            manager.fire(
                state,
                EventWithStats::with_current_time(
                    Event::UpdateUserStats {
                        name: Cow::Owned(format!("{}-absolute", self.name())),
                        value: UserStats::new(
                            UserStatsValue::String(Cow::Owned(stringified_absolute)),
                            AggregatorOps::Avg,
                        ),
                        phantom: PhantomData,
                    },
                    *state.executions(),
                ),
            )?;
        }
        Ok(false)
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

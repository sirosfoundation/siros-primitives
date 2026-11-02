//! The hardware counters of the measuring thread: one perf_event group of user-mode cycles,
//! instructions and branch misses, on the CPU that the thread pins itself to.

/// The events of one counted region.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Counts {
    pub(crate) cycles: u64,
    pub(crate) instructions: u64,
    pub(crate) branch_misses: u64,
}

#[cfg(target_os = "linux")]
pub(crate) use linux::{Counters, pin};
#[cfg(not(target_os = "linux"))]
pub(crate) use other::{Counters, pin};

#[cfg(target_os = "linux")]
mod linux {
    use std::io;
    use std::time::Duration;

    use nix::sched::{CpuSet, sched_setaffinity};
    use nix::unistd::Pid;
    use perf_event::events::Hardware;
    use perf_event::{Builder, Counter, Group};

    use super::Counts;
    use crate::error::Error;

    /// Pins the calling thread to `cpu`.
    pub(crate) fn pin(cpu: usize) -> Result<(), Error> {
        let failure = |source: nix::Error| Error::Pin {
            cpu,
            source: io::Error::from(source),
        };
        let mut set = CpuSet::new();
        set.set(cpu).map_err(failure)?;
        // Pid 0 is the calling thread.
        sched_setaffinity(Pid::from_raw(0), &set).map_err(failure)
    }

    /// One perf_event group on the calling thread and any CPU, which counts user-mode cycles,
    /// instructions and branch misses.
    pub(crate) struct Counters {
        group: Group,
        cycles: Counter,
        instructions: Counter,
        branch_misses: Counter,
        /// The times enabled and running of the group at its last read.
        times: (Duration, Duration),
    }

    impl Counters {
        /// Opens the group. A builder of `perf-event2` counts the calling thread on any CPU in
        /// user mode, without kernel and hypervisor.
        pub(crate) fn open() -> Result<Self, Error> {
            let failure = |source: io::Error| Error::Counters { source };
            let mut leader = Group::builder();
            // A pinned group counts whenever its thread runs, or fails its read.
            let mut group = leader.pinned(true).build_group().map_err(failure)?;
            let mut add = |event: Hardware| group.add(&Builder::new(event)).map_err(failure);
            let cycles = add(Hardware::CPU_CYCLES)?;
            let instructions = add(Hardware::INSTRUCTIONS)?;
            let branch_misses = add(Hardware::BRANCH_MISSES)?;
            Ok(Self {
                group,
                cycles,
                instructions,
                branch_misses,
                times: (Duration::ZERO, Duration::ZERO),
            })
        }

        /// The value of `region` and the events that it counts. A region during which the
        /// kernel shared the counters with other groups is a [`Error::Multiplexed`].
        pub(crate) fn count<T>(
            &mut self,
            region: impl FnOnce() -> T,
        ) -> Result<(T, Counts), Error> {
            let failure = |source: io::Error| Error::Counters { source };
            self.group.reset().map_err(failure)?;
            self.group.enable().map_err(failure)?;
            let value = region();
            self.group.disable().map_err(failure)?;
            let data = self.group.read().map_err(failure)?;
            let times = data
                .time_enabled()
                .zip(data.time_running())
                .ok_or_else(|| failure(io::Error::other("a read without its times")))?;
            let (enabled, running) = times
                .0
                .checked_sub(self.times.0)
                .zip(times.1.checked_sub(self.times.1))
                .ok_or_else(|| failure(io::Error::other("times that run backwards")))?;
            self.times = times;
            if enabled != running {
                return Err(Error::Multiplexed { enabled, running });
            }
            let value_of = |counter: &Counter| {
                data.get(counter)
                    .map(|entry| entry.value())
                    .ok_or_else(|| failure(io::Error::other("a read without a member")))
            };
            Ok((
                value,
                Counts {
                    cycles: value_of(&self.cycles)?,
                    instructions: value_of(&self.instructions)?,
                    branch_misses: value_of(&self.branch_misses)?,
                },
            ))
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod other {
    use super::Counts;
    use crate::error::Error;

    /// Pinning needs Linux.
    pub(crate) fn pin(_cpu: usize) -> Result<(), Error> {
        Err(Error::NoPerfEvent)
    }

    /// The counters of a target without perf_event: none can open.
    pub(crate) enum Counters {}

    impl Counters {
        pub(crate) fn open() -> Result<Self, Error> {
            Err(Error::NoPerfEvent)
        }

        pub(crate) fn count<T>(
            &mut self,
            _region: impl FnOnce() -> T,
        ) -> Result<(T, Counts), Error> {
            match *self {}
        }
    }
}

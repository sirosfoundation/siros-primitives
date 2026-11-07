//! The hardware counters of the measuring thread: one perf_event group of user-mode cycles,
//! instructions and branch misses, on the CPU that the thread pins itself to.

/// The events of one counted region.
#[derive(Clone, Copy, Debug)]
pub struct Counts {
    /// Core cycles.
    pub cycles: u64,
    /// Retired instructions.
    pub instructions: u64,
    /// Mispredicted branches.
    pub branch_misses: u64,
}

#[cfg(target_os = "linux")]
pub use linux::{Counters, pin, pin_thread};
#[cfg(not(any(target_os = "linux", target_arch = "wasm32")))]
pub use other::{Counters, pin};
#[cfg(target_arch = "wasm32")]
pub(crate) use page::Counters;

/// The counters of a WebAssembly bench: the driver counts the thread that runs the module, and
/// the page passes each read to it.
#[cfg(target_arch = "wasm32")]
mod page {
    use super::Counts;
    use crate::error::Error;
    use crate::host;

    /// The driver's counters, which the module reaches through its imports.
    pub(crate) struct Counters {
        /// The cycles that a count leaves out.
        empty: u64,
    }

    impl Counters {
        /// The driver opened the counters before the module started.
        #[expect(
            clippy::unnecessary_wraps,
            reason = "the native counters fail to open; the sweep takes either"
        )]
        pub(crate) fn open() -> Result<Self, Error> {
            Ok(Self { empty: 0 })
        }

        /// Leaves `empty` cycles, the fixed cost of a count, out of the later counts.
        pub(crate) fn leave_out(&mut self, empty: u64) {
            self.empty = empty;
        }

        /// The value of `region` and the events that it counts. The driver fails the page's call
        /// on a region during which the kernel shared the counters with other groups. A region of
        /// few operations can count fewer cycles than the fixed cost, and then counts none.
        #[expect(
            clippy::unnecessary_wraps,
            reason = "the native counters fail a read; the sweep takes either"
        )]
        pub(crate) fn count(
            &mut self,
            region: &mut dyn FnMut() -> usize,
        ) -> Result<(usize, Counts), Error> {
            host::begin();
            let value = region();
            let counts = host::end();
            Ok((
                value,
                Counts {
                    cycles: counts.cycles.saturating_sub(self.empty),
                    ..counts
                },
            ))
        }
    }
}

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
    pub fn pin(cpu: usize) -> Result<(), Error> {
        // Tid 0 is the calling thread.
        pin_thread(0, cpu)
    }

    /// Pins the thread `tid` to `cpu`.
    pub fn pin_thread(tid: i32, cpu: usize) -> Result<(), Error> {
        let failure = |source: nix::Error| Error::Pin {
            cpu,
            source: io::Error::from(source),
        };
        let mut set = CpuSet::new();
        set.set(cpu).map_err(failure)?;
        sched_setaffinity(Pid::from_raw(tid), &set).map_err(failure)
    }

    /// One perf_event group on one thread and any CPU, which counts user-mode cycles,
    /// instructions and branch misses.
    pub struct Counters {
        group: Group,
        cycles: Counter,
        instructions: Counter,
        branch_misses: Counter,
        /// The times enabled and running of the group at its last read.
        times: (Duration, Duration),
    }

    impl Counters {
        /// Opens the group on the calling thread.
        pub fn open() -> Result<Self, Error> {
            // Pid 0 is the calling thread.
            Self::build(0)
        }

        /// Opens the group on the thread `tid` of another process, which this process may trace.
        pub fn observe(tid: i32) -> Result<Self, Error> {
            Self::build(tid)
        }

        /// Opens the group on the thread `tid`. A builder of `perf-event2` counts on any
        /// CPU in user mode, without kernel and hypervisor.
        fn build(tid: i32) -> Result<Self, Error> {
            let failure = |source: io::Error| Error::Counters { source };
            // A pinned group counts whenever its thread runs, or fails its read.
            let mut group = Group::builder()
                .observe_pid(tid)
                .pinned(true)
                .build_group()
                .map_err(failure)?;
            let mut add = |event: Hardware| {
                group
                    .add(Builder::new(event).observe_pid(tid))
                    .map_err(failure)
            };
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
        pub fn count(
            &mut self,
            region: &mut dyn FnMut() -> usize,
        ) -> Result<(usize, Counts), Error> {
            self.begin()?;
            let value = region();
            Ok((value, self.end()?))
        }

        /// Resets and enables the group.
        pub fn begin(&mut self) -> Result<(), Error> {
            let failure = |source: io::Error| Error::Counters { source };
            self.group.reset().map_err(failure)?;
            self.group.enable().map_err(failure)
        }

        /// Disables the group and returns what it counted since [`begin`](Self::begin). A span
        /// during which the kernel shared the counters with other groups is a
        /// [`Error::Multiplexed`].
        pub fn end(&mut self) -> Result<Counts, Error> {
            let failure = |source: io::Error| Error::Counters { source };
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
            Ok(Counts {
                cycles: value_of(&self.cycles)?,
                instructions: value_of(&self.instructions)?,
                branch_misses: value_of(&self.branch_misses)?,
            })
        }
    }
}

#[cfg(not(any(target_os = "linux", target_arch = "wasm32")))]
mod other {
    use super::Counts;
    use crate::error::Error;

    /// Pinning needs Linux.
    pub fn pin(_cpu: usize) -> Result<(), Error> {
        Err(Error::NoPerfEvent)
    }

    /// The counters of a target without perf_event: none can open.
    pub enum Counters {}

    impl Counters {
        /// None can open.
        pub fn open() -> Result<Self, Error> {
            Err(Error::NoPerfEvent)
        }

        /// None can open.
        pub fn count(
            &mut self,
            _region: &mut dyn FnMut() -> usize,
        ) -> Result<(usize, Counts), Error> {
            match *self {}
        }
    }
}

use core::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use alloc::boxed::Box;

use crate::{
    drivers::timer::{Instant, schedule_preempt},
    kernel::cpu_id::CpuId,
    process::{TaskState, owned::OwnedTask},
};

use super::{DEFAULT_TIME_SLICE, SCHED_WEIGHT_BASE, VT_FIXED_SHIFT};

pub struct SchedulableTask {
    pub task: Box<OwnedTask>,
    pub v_runtime: u128,
    /// Virtual time at which the task becomes eligible (v_ei).
    pub v_eligible: u128,
    /// Virtual deadline (v_di) used by the EEVDF scheduler.
    pub v_deadline: u128,
    pub exec_start: Option<Instant>,
    pub deadline: Option<Instant>,
    pub last_run: Option<Instant>,
}

impl Deref for SchedulableTask {
    type Target = OwnedTask;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}

impl DerefMut for SchedulableTask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}

impl SchedulableTask {
    pub fn new(task: Box<OwnedTask>) -> Box<Self> {
        Box::new(Self {
            task,
            v_runtime: 0,
            v_eligible: 0,
            v_deadline: 0,
            exec_start: None,
            deadline: None,
            last_run: None,
        })
    }

    /// Update accounting info for this task given the latest time.
    pub fn tick(&mut self, now: Instant) {
        let delta_vt = if let Some(start) = self.exec_start {
            let delta = now - start;
            let w = self.weight() as u128;
            let dv = ((delta.as_nanos() as u128) << VT_FIXED_SHIFT) / w;
            self.v_runtime = dv;
            dv
        } else {
            0
        };

        // Advance its eligible time by the virtual run time it just used
        // (EEVDF: v_ei += t_used / w_i).
        self.v_eligible += delta_vt;

        // Re-issue a virtual deadline
        let q_ns: u128 = DEFAULT_TIME_SLICE.as_nanos();
        let v_delta = (q_ns << VT_FIXED_SHIFT) / self.weight() as u128;
        let v_ei = self.v_eligible;
        self.v_deadline = v_ei + v_delta;
    }

    /// Compute this task's scheduling weight.
    ///
    /// weight = priority + SCHED_WEIGHT_BASE
    /// The sum is clamped to a minimum of 1
    pub fn weight(&self) -> u32 {
        let w = self.task.priority as i32 + SCHED_WEIGHT_BASE;
        if w <= 0 { 1 } else { w as u32 }
    }

    pub fn compare_with(&self, other: &Self) -> core::cmp::Ordering {
        if self.is_idle_task() {
            return Ordering::Greater;
        }

        if other.is_idle_task() {
            return Ordering::Less;
        }

        self.v_deadline
            .cmp(&other.v_deadline)
            .then_with(|| self.v_runtime.cmp(&other.v_runtime))
            // If completely equal, prefer the one that hasn't run in a while?
            // Or prefer the one already running to avoid cache thrashing?
            // Usually irrelevant for EEVDF but strict ordering is good for
            // stability.
            .then_with(|| match (self.last_run, other.last_run) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            })
    }

    /// Update accounting information when the task is about to be inserted into
    /// a runqueue.
    pub fn inserting_into_runqueue(&mut self, vclock: u128) {
        // A freshly enqueued task becomes eligible immediately.
        self.v_eligible = vclock;

        // Grant it an initial virtual deadline proportional to its weight.
        let q_ns: u128 = DEFAULT_TIME_SLICE.as_nanos();
        let v_delta = (q_ns << VT_FIXED_SHIFT) / self.weight() as u128;
        let new_v_deadline = vclock + v_delta;
        self.v_deadline = new_v_deadline;

        // Since the task is not executing yet, its exec_start must be `None`.
        self.exec_start = None;
    }

    /// Setup task accounting info such that it is about to be executed.
    pub fn about_to_execute(&mut self, now: Instant) {
        self.exec_start = Some(now);
        *self.last_cpu.lock_save_irq() = CpuId::this();
        *self.state.lock_save_irq() = TaskState::Running;

        // Deadline logic
        if self.deadline.is_none_or(|d| d <= now + DEFAULT_TIME_SLICE) {
            self.deadline = Some(now + DEFAULT_TIME_SLICE);
        }

        if let Some(d) = self.deadline {
            schedule_preempt(d);
        }
    }
}

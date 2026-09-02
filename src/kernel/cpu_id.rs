use libkernel::CpuOps;

use crate::arch::ArchImpl;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CpuId(usize);

impl CpuId {
    pub fn this() -> CpuId {
        CpuId(ArchImpl::id())
    }

    pub fn from_value(id: usize) -> Self {
        Self(id)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

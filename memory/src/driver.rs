use crate::types::{DriverFunctions, Memory, ReadResult, UPtr};

pub struct Driver {}

impl Driver {
    pub const fn uninitialized() -> Self {
        Self {}
    }
}

#[allow(unused)]
impl DriverFunctions for Driver {
    unsafe fn new(pid: u32) -> ReadResult<Self>
    where
        Self: Sized,
    {
        todo!("Add your own driver for read and write");
    }

    unsafe fn get_process_base(&mut self) -> ReadResult<UPtr> {
        todo!("Add your own driver for process base");
    }

    unsafe fn read(&mut self, data: Memory) -> ReadResult<()> {
        todo!("Add your own driver for read");
    }

    unsafe fn write(&mut self, data: Memory) -> ReadResult<()> {
        todo!("Add your own driver for write");
    }

    unsafe fn find_guard(&mut self) -> ReadResult<UPtr> {
        todo!("Add your own driver for guard");
    }
}

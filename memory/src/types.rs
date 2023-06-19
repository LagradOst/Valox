use memory_macros::c_class;

use crate::memory::{read, read_array};

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryError {
    InvalidArg,
    InvalidAdress,
    BadRead,
    BadData,
    BadGuard,
    BadProcessBase
    //Nt(i32),
    //Message(&'static str)
}

#[c_class]
pub(crate) struct Memory {
    pub(crate) target_address: UPtr,
    pub(crate) buffer_address: UPtr,
    pub(crate) buffer_size: usize,
}

pub type ReadResult<T> = Result<T, MemoryError>;
pub type UPtr = u64;

pub(crate) trait DriverFunctions {
    unsafe fn new(pid: u32) -> ReadResult<Self>
    where
        Self: Sized;
    unsafe fn get_process_base(&mut self) -> ReadResult<UPtr>;
    unsafe fn read(&mut self, data: Memory) -> ReadResult<()>;
    unsafe fn write(&mut self, data: Memory) -> ReadResult<()>;
    unsafe fn find_guard(&mut self) -> ReadResult<UPtr>;
}

#[c_class]
#[derive(Hash)]
pub struct Ptr<T> {
    pub ptr: UPtr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::ops::Add<usize> for Ptr<T> {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self {
            ptr: self.ptr + (other * std::mem::size_of::<T>()) as u64,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Ptr<T> {
    pub const fn uninitialized() -> Self {
        Self {
            ptr: 0,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new(ptr: UPtr) -> Self {
        Self {
            ptr,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn ptr<V>(other: Ptr<V>) -> Self {
        Self {
            ptr: other.ptr,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::default::Default for Ptr<T> {
    fn default() -> Self {
        Self::uninitialized()
    }
}

/** simple read wrapper for T* that is used for arrays, syntactic sugar*/
#[c_class]
pub struct PtrArray<T> {
    pub ptr: UPtr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> PtrArray<T> {
    pub const fn uninitialized() -> Self {
        Self {
            ptr: 0,
            phantom: std::marker::PhantomData,
        }
    }
    pub fn get_address(&self, index: usize) -> UPtr {
        self.ptr + (index * std::mem::size_of::<T>()) as UPtr
    }

    pub fn get_ptr(&self, index: usize) -> Ptr<T> {
        Ptr {
            ptr: self.get_address(index),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn index(&self, index: usize) -> ReadResult<T> {
        read::<T>(self.get_address(index))
    }

    pub fn take(&self, size : usize) -> ReadResult<Vec<T>> {
        read_array(self.ptr, size)
    }
}

impl<T> Ptr<T> {
    pub fn read(&self) -> ReadResult<T> {
        self.validate()?;
        read(self.ptr)
    }

    pub fn cache_read(&self) -> ReadResult<T> {
        self.validate()?;
        read(self.ptr)
    }
}

impl<T> IsValid for Ptr<T> {
    fn is_valid(&self) -> bool {
        self.ptr.is_valid()
    }
}

impl<T: IsValid> Ptr<T> {
    pub fn readv(&self) -> ReadResult<T> {
        self.validate()?;
        let value: T = read(self.ptr)?;
        value.validate()?;
        Ok(value)
    }
}

impl<T> IsValid for PtrArray<T> {
    fn is_valid(&self) -> bool {
        self.ptr.is_valid()
    }
}

pub trait IsValid {
    fn is_valid(&self) -> bool;
    fn is_invalid(&self) -> bool {
        !self.is_valid()
    }
    fn validate(&self) -> Result<&Self, MemoryError> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(MemoryError::BadData)
        }
    }
}

impl IsValid for u64 {
    fn is_valid(&self) -> bool {
        *self != 0xCCCCCCCCCCCCCCCC && *self > 0x10000 && *self <= 0xFFFFFFFFFF000000
    }
}

pub trait TryCast {
    const HASH: u64;
}
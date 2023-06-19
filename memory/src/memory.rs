use std::{ffi::OsString, mem::MaybeUninit, os::windows::prelude::OsStringExt, sync::Mutex};

use winapi::um::{
    handleapi::CloseHandle,
    tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
};

use crate::{
    types::{DriverFunctions, IsValid, Memory, MemoryError, ReadResult, UPtr},
    Driver,
};

pub unsafe fn get_process_id(name: &str) -> Option<u32> {
    let mut process: PROCESSENTRY32W = MaybeUninit::zeroed().assume_init();
    process.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

    if Process32FirstW(snapshot, &mut process) == 0 {
        CloseHandle(snapshot);
        return None;
    }

    while Process32NextW(snapshot, &mut process) != 0 {
        let process_name = OsString::from_wide(&process.szExeFile);
        let Some(process_name_str) = process_name.to_str() else {
            continue;
        };
        if process_name_str.contains(name) {
            CloseHandle(snapshot);
            return Some(process.th32ProcessID);
        }
    }

    CloseHandle(snapshot);
    None
}

static mut HAS_INITALIZED: Mutex<bool> = Mutex::new(false);
static mut DRIVER: Driver = Driver::uninitialized();
static mut GUARD: UPtr = 0;
static mut PROCESS_BASE: UPtr = 0;

pub fn guard() -> UPtr {
    unsafe { GUARD }
}

pub fn process_base() -> UPtr {
    unsafe { PROCESS_BASE }
}

pub fn init_memory(process_name: &str) -> ReadResult<()> {
    unsafe {
        let mut initalized = HAS_INITALIZED.lock().unwrap();
        // already initalized
        if *initalized {
            return Ok(());
        }

        let Some(pid) = get_process_id(process_name) else  {
            eprintln!("Unable to find process {process_name}, have you started it?");
            return Err(MemoryError::InvalidArg);
        };

        let mut driver = Driver::new(pid)?;

        GUARD = driver.find_guard()?;
        PROCESS_BASE = driver.get_process_base()?;
        DRIVER = driver;
        *initalized = true;
        Ok(())
    }
}

fn is_guarded(adress: u64) -> bool {
    let result = adress & 0xFFFFFFF000000000u64;
    result == 0x8000000000u64 || result == 0x10000000000u64
}

pub fn validate_ptr(address: u64) -> u64 {
    if is_guarded(address) {
        guard() + (address & 0xFFFFFFu64)
    } else {
        address
    }
}

pub fn read<T>(address: UPtr) -> ReadResult<T> {
    unsafe {
        if address.is_invalid() {
            return ReadResult::Err(MemoryError::InvalidAdress);
        }

        let mut buffer: T = std::mem::MaybeUninit::zeroed().assume_init();

        DRIVER.read(Memory {
            target_address: validate_ptr(address),
            buffer_address: (&mut buffer as *mut T) as UPtr,
            buffer_size: std::mem::size_of::<T>(),
        })?;

        Ok(buffer)
    }
}

pub fn read_array<T>(address: UPtr, size: usize) -> ReadResult<Vec<T>> {
    unsafe {
        if address.is_invalid() {
            return ReadResult::Err(MemoryError::InvalidAdress);
        }

        let mut buffer: Vec<T> = Vec::with_capacity(size);

        DRIVER.read(Memory {
            target_address: validate_ptr(address),
            buffer_address: buffer.as_mut_ptr() as UPtr,
            buffer_size: std::mem::size_of::<T>() * size,
        })?;

        buffer.set_len(size);

        Ok(buffer)
    }
}

// this is called writef instead of write to not missmatch with the useful macro write! from the std
pub fn writef<T>(address: UPtr, value: T) -> ReadResult<()> {
    unsafe {
        if address.is_invalid() {
            return ReadResult::Err(MemoryError::InvalidAdress);
        }

        DRIVER.write(Memory {
            target_address: validate_ptr(address),
            buffer_address: (&value as *const T) as UPtr,
            buffer_size: std::mem::size_of::<T>(),
        })?;

        Ok(())
    }
}

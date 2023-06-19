#![feature(try_blocks)]
use classes::*;
use ue4_rs::prelude::*;

mod bones;
mod hack;
mod classes;

pub fn initalize() -> ReadResult<()> {
    init_memory("VALORANT-Win64-Shipping.exe")?;
    unsafe {
        let a = process_base() + 0x90e0600;

        FNAME_POOL_PTR = a;
        VALORANT_KEY = read(guard())?;
    }
    assert_eq!(
        get_fname(0),
        Some("None".to_owned()),
        "Expected fname(0) to be None, update your fname offset"
    );
    Ok(())
}

pub fn gworld() -> ReadResult<UWorldPtr> {
    read::<UWorldPtr>(guard() + 0x60)
}

fn main() -> ReadResult<()> {
    hack::start_hack()
}

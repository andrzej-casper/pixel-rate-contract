#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{string::String, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{URef, EntryPoint, EntryPoints, EntryPointAccess, EntryPointType, CLType};
use core::convert::TryInto;

const MOVIE_ARG_NAME: &str = "movie";
const RATING_ARG_NAME: &str = "rating";

const ENTRY_POINT_RATE_MOVIE: &str = "rate_movie";

// Entry point that stores movie rating in named key.
#[no_mangle]
pub extern "C" fn rate_movie() {
    let movie: String = runtime::get_named_arg(MOVIE_ARG_NAME);
    let rating: String = runtime::get_named_arg(RATING_ARG_NAME);

    match runtime::get_key(&movie) {
        None => {
            let key = storage::new_uref(rating).into();
            runtime::put_key(&movie, key);
        }
        Some(_key) => {
            //Get the URref of the named key
            let key: URef = _key.try_into().unwrap_or_revert();
            storage::write(key, rating);
        }
    }
}

#[no_mangle]
pub extern "C" fn call() {
    // Define entrypoints - one session code for rating movie.
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_RATE_MOVIE,
        Vec::new(), // just info???
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    // Install as non-upgradable contract.
    let (contract_hash, _) = storage::new_locked_contract(entry_points, None, None, None);

    // Store a named key with the contract hash (under current account).
    runtime::put_key("contract_hash", contract_hash.into());
}

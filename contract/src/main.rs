#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::string::String;
use alloc::vec;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, URef,
};

// Entry point that stores movie rating in named key.
const ENTRY_POINT_RATE_MOVIE: &str = "rate_movie";
const MOVIE_ARG_NAME: &str = "movie";
const RATING_ARG_NAME: &str = "rating";
//
#[no_mangle]
pub extern "C" fn rate_movie() {
    // Parse arguments - movie name and rating.
    let movie: String = runtime::get_named_arg(MOVIE_ARG_NAME);
    let rating: u8 = runtime::get_named_arg(RATING_ARG_NAME);

    // Get or create map for particular movie.
    let movie_map_uref: URef = match runtime::get_key(&movie) {
        None => storage::new_dictionary(&movie).unwrap_or_revert(),
        Some(k) => *k.as_uref().unwrap_or_revert(),
    };

    // Store corresponding account hash and given rating.
    let caller = base16::encode_lower(&runtime::get_caller().value());
    storage::dictionary_put(movie_map_uref, &caller, rating);
}

// Contract setup.
const CONTRACT_HASH_KEY: &str = "pixel_rate_contract_hash";
//
#[no_mangle]
pub extern "C" fn call() {
    // Define entrypoints - one session code for rating movie.
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_RATE_MOVIE,
        vec![
            Parameter::new(MOVIE_ARG_NAME, String::cl_type()),
            Parameter::new(RATING_ARG_NAME, u8::cl_type()),
        ], // Seems to be NOT validated, used only as guidance.
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract, // We want to store data is contract context.
    ));

    // Install as non-upgradable contract.
    let (contract_hash, _) = storage::new_locked_contract(entry_points, None, None, None);

    // Store a named key with the contract hash (under current account).
    runtime::put_key(CONTRACT_HASH_KEY, contract_hash.into());
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
        DEFAULT_PAYMENT,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash, runtime_args, CLValue, Key, Motes, PublicKey, RuntimeArgs, SecretKey,
        StoredValue, U512,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    const CONTRACT_WASM: &str = "../../contract/target/wasm32-unknown-unknown/release/contract.wasm";

    fn setup() -> InMemoryWasmTestBuilder {
        // Create keypair.
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);
        // Create a GenesisAccount.
        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);

        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // The test framework checks for compiled Wasm files in '<current working dir>/wasm'.  Paths
        // relative to the current working dir (e.g. 'wasm/contract.wasm') can also be used, as can
        // absolute paths.

        // install contract.wasm
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {};

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        builder
    }

    fn call_contract(
        builder: &mut InMemoryWasmTestBuilder,
        account_addr: AccountHash,
        movie: &str,
        rating: u8,
    ) {
        let session_args = runtime_args! {
            "movie" => movie,
            "rating" => rating,
        };
        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_stored_session_named_key("contract_hash", "rate_movie", session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
        builder.exec(execute_request).commit().expect_success();
    }

    #[test]
    fn should_add_non_existing_element() {
       let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
       let public_key = PublicKey::from(&secret_key);

       // Create an AccountHash from a public key.
       let account_addr = AccountHash::from(&public_key);

       let mut builder = setup();

       call_contract(&mut builder, account_addr, "the-godfather", 4 as u8);

       //get account
       let account = builder
           .query(None, Key::Account(account_addr), &[])
           .expect("should query account")
           .as_account()
           .cloned()
           .expect("should be account");
    
        let retvaluekey = *(account
            .named_keys()
            .get("contract_hash")
            .expect("named key should exist"));

        let retvalue = builder
            .query(None, retvaluekey, &[])
            .expect("Value should exist");
        
        let contract = retvalue.as_contract().unwrap();
        let movie_map: Key = *contract.named_keys().get("the-godfather").expect("should have key");

        let account_addr_raw = base16::encode_lower(&account_addr.value());
        let rating = builder.query_dictionary_item(None, *movie_map.as_uref().unwrap(), &account_addr_raw).unwrap();
        let expected_output: u8 = 4;
        assert_eq!(
          rating,
          StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
            "Should have valid rating"
        );
    }

}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}

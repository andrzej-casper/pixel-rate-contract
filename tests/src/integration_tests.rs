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
    const CONTRACT_WASM: &str = "contract.wasm";

    fn setup(named_key: &str) -> InMemoryWasmTestBuilder {
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
        let empty_list: Vec<&str> = Vec::new();
        let session_args = runtime_args! {
            "movie" => named_key,
            //"keys" => empty_list,
            //"method" => String::from("add")
        };

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
        named_key: &str,
        data: Vec<&str>,
        method: &str,
    ) {
        let session_code = PathBuf::from(CONTRACT_WASM);

        let session_args = runtime_args! {
            "movie" => named_key,
            //"keys" => data,
            //"method" => String::from(method)
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
        builder.exec(execute_request).commit().expect_success();
    }

    fn call_contract_missing_parameter(
        builder: &mut InMemoryWasmTestBuilder,
        account_addr: AccountHash,
        named_key: &str,
        data: Vec<&str>,
    ) {
        let session_code = PathBuf::from(CONTRACT_WASM);

        let session_args = runtime_args! {
            //"named-key" => named_key,
            //"keys" => data
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();
        builder.exec(execute_request).commit().expect_success();
    }
    #[test]
    fn should_create_named_keys() {
        let named_key = "my-named-key";
        let builder = setup(named_key);
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);

        //get account
        let account = builder
            .query(None, Key::Account(account_addr), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        let retvaluekey = *(account
            .named_keys()
            .get(named_key)
            .expect("named key should exist"));
        
        let retvalue = builder
            .query(None, retvaluekey, &[])
            .expect("Value should exist");

        // make assertions
        let expected_output: Vec<&str> = Vec::new();
        assert_eq!(
            retvalue,
            StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
            "Value should be empty"
        );
    }

    #[test]
    #[should_panic(expected = "ApiError::MissingArgument")]
    fn should_panic_missing_parameters() {
        let named_key = "my-named-key";
        let mut builder = setup(named_key);
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);

        let data: Vec<&str> = Vec::new();

        call_contract_missing_parameter(&mut builder, account_addr, named_key, data);
    }

    //#[test]
    //fn should_add_non_existing_element() {
    //    let named_key = "my-named-key";
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data: Vec<&str> = Vec::new();
    //    data.push("TEST");

    //    let mut builder = setup(named_key);

    //    call_contract(&mut builder, account_addr, named_key, data, "add");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("TEST");
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Should contain 1 element"
    //    );
    //}

    //#[test]
    //fn should_add_multiple_elements() {
    //    let named_key = "my-named-key";
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data: Vec<&str> = Vec::new();
    //    data.push("ID1;VALUE");
    //    data.push("ID2;VALUE");
    //    data.push("ID3;VALUE");

    //    let mut builder = setup(named_key);

    //    call_contract(&mut builder, account_addr, named_key, data, "add");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("ID1;VALUE");
    //    expected_output.push("ID2;VALUE");
    //    expected_output.push("ID3;VALUE");

    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Should contain 3 elements"
    //    );
    //}

    //#[test]
    //fn should_update_existing_element() {
    //    let named_key = "my-named-key";
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data_call_one: Vec<&str> = Vec::new();
    //    data_call_one.push("ID1;VALUE");

    //    let mut builder = setup(named_key);

    //    call_contract(&mut builder, account_addr, named_key, data_call_one, "add");

    //    let mut data_call_two: Vec<&str> = Vec::new();
    //    data_call_two.push("ID1;VALUE2");

    //    call_contract(&mut builder, account_addr, named_key, data_call_two, "add");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("ID1;VALUE2");
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Should be updated"
    //    );
    //}

    //#[test]
    //fn should_del_element() {
    //    let named_key = "my-named-key";
    //    let mut builder = setup(named_key);
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data_to_add: Vec<&str> = Vec::new();
    //    data_to_add.push("ID1;VALUE");
    //    let mut data_to_remove: Vec<&str> = Vec::new();
    //    data_to_remove.push("ID1;VALUE");
    //    call_contract(&mut builder, account_addr, named_key, data_to_add, "add");
    //    call_contract(&mut builder, account_addr, named_key, data_to_remove, "del");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let expected_output: Vec<&str> = Vec::new();
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Value should be empty"
    //    );
    //}

    //#[test]
    //fn should_del_multiple_elements() {
    //    let named_key = "my-named-key";
    //    let mut builder = setup(named_key);
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data_to_add: Vec<&str> = Vec::new();
    //    data_to_add.push("ID1;VALUE");
    //    data_to_add.push("ID2;VALUE");
    //    data_to_add.push("ID3;VALUE");
    //    let mut data_to_remove: Vec<&str> = Vec::new();
    //    data_to_remove.push("ID1;VALUE");
    //    data_to_remove.push("ID2;VALUE");
    //    // expected_output.push("ID3;VALUE");
    //    call_contract(&mut builder, account_addr, named_key, data_to_add, "add");
    //    call_contract(&mut builder, account_addr, named_key, data_to_remove, "del");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("ID3;VALUE");
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Value should be empty"
    //    );
    //}

    //#[test]
    //fn should_not_del_element_non_existing_element() {
    //    let named_key = "my-named-key";
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data_call_one: Vec<&str> = Vec::new();
    //    data_call_one.push("ID1;VALUE");

    //    let mut builder = setup(named_key);

    //    call_contract(&mut builder, account_addr, named_key, data_call_one, "add");

    //    let mut data_call_two: Vec<&str> = Vec::new();
    //    data_call_two.push("ID2;VALUE");

    //    call_contract(&mut builder, account_addr, named_key, data_call_two, "del");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("ID1;VALUE");
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Value should contain 1 element"
    //    );
    //}

    //#[test]
    //fn should_remove_all_elements() {
    //    let named_key = "my-named-key";
    //    let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
    //    let public_key = PublicKey::from(&secret_key);

    //    // Create an AccountHash from a public key.
    //    let account_addr = AccountHash::from(&public_key);

    //    let mut data_call: Vec<&str> = Vec::new();
    //    data_call.push("ID1;VALUE");
    //    data_call.push("ID2;VALUE");

    //    let mut builder = setup(named_key);

    //    call_contract(&mut builder, account_addr, named_key, data_call, "add");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let mut expected_output: Vec<&str> = Vec::new();
    //    expected_output.push("ID1;VALUE");
    //    expected_output.push("ID2;VALUE");
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Value should contain 2 elements"
    //    );

    //    let data_call: Vec<&str> = Vec::new();

    //    call_contract(&mut builder, account_addr, named_key, data_call, "delall");

    //    //get account
    //    let account = builder
    //        .query(None, Key::Account(account_addr), &[])
    //        .expect("should query account")
    //        .as_account()
    //        .cloned()
    //        .expect("should be account");

    //    let retvaluekey = *(account
    //        .named_keys()
    //        .get(named_key)
    //        .expect("named key should exist"));

    //    let retvalue = builder
    //        .query(None, retvaluekey, &[])
    //        .expect("Value should exist");

    //    //
    //    // // make assertions
    //    let expected_output: Vec<&str> = Vec::new();
    //    assert_eq!(
    //        retvalue,
    //        StoredValue::CLValue(CLValue::from_t(expected_output).unwrap()),
    //        "Value should contain 0 elements"
    //    );
    //}
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}

use agent_storage_client::{AgentStorageClient, AgentStorageClientCtors, agent_storage::*};
use sails_rs::{client::*, gtest::*};

const ACTOR_ID: u64 = 42;

#[tokio::test]
async fn storage_put_and_get() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let program_code_id = system.submit_code(agent_storage::WASM_BINARY);

    let env = GtestEnv::new(system, ACTOR_ID.into());

    let program = env
        .deploy::<agent_storage_client::AgentStorageClientProgram>(
            program_code_id,
            b"salt".to_vec(),
        )
        .create()
        .await
        .unwrap();

    let mut service_client = program.agent_storage();

    let put_result = service_client
        .put("test_key".into(), b"test_value".to_vec(), 1000)
        .await
        .unwrap();
    assert!(put_result.is_ok());

    let get_result = service_client
        .get("test_key".into())
        .await
        .unwrap();
    assert_eq!(get_result, Some(b"test_value".to_vec()));
}

#[tokio::test]
async fn storage_keys() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let program_code_id = system.submit_code(agent_storage::WASM_BINARY);

    let env = GtestEnv::new(system, ACTOR_ID.into());

    let program = env
        .deploy::<agent_storage_client::AgentStorageClientProgram>(
            program_code_id,
            b"salt2".to_vec(),
        )
        .create()
        .await
        .unwrap();

    let mut service_client = program.agent_storage();

    service_client.put("key1".into(), b"val1".to_vec(), 1000).await.unwrap();
    service_client.put("key2".into(), b"val2".to_vec(), 1000).await.unwrap();

    let keys = service_client.keys().await.unwrap();
    assert!(keys.contains(&"key1".into()));
    assert!(keys.contains(&"key2".into()));
}

#[tokio::test]
async fn storage_remove() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);
    let program_code_id = system.submit_code(agent_storage::WASM_BINARY);

    let env = GtestEnv::new(system, ACTOR_ID.into());

    let program = env
        .deploy::<agent_storage_client::AgentStorageClientProgram>(
            program_code_id,
            b"salt3".to_vec(),
        )
        .create()
        .await
        .unwrap();

    let mut service_client = program.agent_storage();

    service_client.put("removeme".into(), b"value".to_vec(), 1000).await.unwrap();

    let removed = service_client.remove("removeme".into()).await.unwrap();
    assert_eq!(removed, Some(b"value".to_vec()));

    let get_after = service_client.get("removeme".into()).await.unwrap();
    assert_eq!(get_after, None);
}
use std::pin::pin;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use goose::logger::GooseLog;
use goose::metrics::TransactionMetric;
use goose::prelude::*;
use goose_eggs::validate_and_load_static_assets;
use once_cell::sync::Lazy;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use xrpl_rust_sdk_client::async_client::AsyncReqwestClient;
use xrpl_rust_sdk_client::ReqwestClient;
use xrpl_rust_sdk_client::traits::JsonRpcClient;
use xrpl_rust_sdk_core::core::codec::base58::tokentype::TokenType::Ed25519Seed;
use xrpl_rust_sdk_core::core::crypto::{PrivateKey as PrivateKeyTrait, PublicKey as PublicKeyTrait, ToFromBase58};
use xrpl_rust_sdk_core::core::crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use xrpl_rust_sdk_core::core::crypto::keys::{PrivateKey, PublicKey};
use xrpl_rust_sdk_core::core::crypto::secp256k1::Secp256k1PrivateKey;
use xrpl_rust_sdk_core::core::crypto::seed::Seed;
use xrpl_rust_sdk_core::core::model::client::account_info::AccountInfoRequest;
use xrpl_rust_sdk_core::core::model::client::JsonRpcRequest;
use xrpl_rust_sdk_core::core::model::client::submit::{SubmitRequest, SubmitResult};
use xrpl_rust_sdk_core::core::model::transactions::cftoken_create::CFTokenCreate;
use xrpl_rust_sdk_core::core::model::transactions::cftoken_issuance_create::CFTokenIssuanceCreate;
use xrpl_rust_sdk_core::core::model::transactions::payment::Payment;
use xrpl_rust_sdk_core::core::model::transactions::TransactionType;
use xrpl_rust_sdk_core::core::types::{ACCOUNT_ZERO, AccountId, Amount, Currency, Hash160, ONE_XRP_IN_DROPS, XrpAmount};
use xrpl_rust_sdk_core::core::types::cft_amount::CftAmount;

const RIPPLED_URL: &'static str = "http://localhost:5005";
const WS_URL: &'static str = "ws://localhost:6006";
static RPC_CLIENT: Lazy<AsyncReqwestClient> = Lazy::new(|| AsyncReqwestClient::new(RIPPLED_URL.to_string()));
const ASSET_CODE: &'static str = "0000000000000000000000005553440000000000";

struct Account {
    account_id: AccountId,
    private_key: PrivateKey,
    public_key: PublicKey,
    destinations: Vec<AccountId>,
    sequence: u32,
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let success_count = Arc::new(RwLock::new(0u64));
    let num_ledgers = Arc::new(RwLock::new(0u64));
    let subscribe_task = tokio::spawn(
        count_validated_transactions(success_count.clone(), num_ledgers.clone())
    );

    let goose_attack = GooseAttack::initialize()?
        .register_scenario(
            Scenario::new("XRP Payments")
                .set_host(RIPPLED_URL)
                .register_transaction(transaction!(create_and_fund_account).set_on_start())
                .register_transaction(transaction!(send_payment))
        )
        .execute();

    let start_time = Instant::now();

    tokio::select! {
        _ = subscribe_task => {
            println!("Websocket Subscription task completed.")
        }
        _ = goose_attack => {
            println!("Goose Attack completed.")
        }
    }

    let elapsed_time = start_time.elapsed();
    println!("success count: {}", *success_count.read().unwrap());
    println!("num ledgers: {}", *num_ledgers.read().unwrap());
    println!("avg tx per ledger: {}", *success_count.read().unwrap() / *num_ledgers.read().unwrap());
    println!("avg tx/s: {}", *success_count.read().unwrap() / elapsed_time.as_secs());
    Ok(())
}

async fn count_validated_transactions(success_count: Arc<RwLock<u64>>, num_ledgers: Arc<RwLock<u64>>) {
    let (mut websocket, _) = connect_async(WS_URL).await.expect("Failed to connect");
    let subscribe_command = r#"{
        "id": 1,
        "command": "subscribe",
        "streams": ["ledger"]
    }"#;
    websocket.send(Message::Text(subscribe_command.into())).await.unwrap();
    while let Some(Ok(message)) = websocket.next().await {
        match message {
            Message::Close(Some(frame)) => {
                // Handle the close message and break the loop
                println!("WebSocket closed: {:?}", frame);
                break;
            }
            Message::Text(t) => {
                serde_json::from_str::<Value>(&t)
                    .map_or_else(
                        |e| println!("Couldn't deserialize websocket message {}.", t),
                        |json| {
                            let tx_count = json
                                .get("txn_count")
                                .map(|txn_count| txn_count.as_u64().unwrap())
                                .unwrap_or_else(|| 0);
                            *success_count.write().unwrap() += tx_count;
                            *num_ledgers.write().unwrap() += 1;
                        },
                    )
            }
            _ => {}
        }
    }

    // Close the WebSocket connection gracefully
    websocket.close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: Default::default(),
    })).await;
}

async fn create_and_fund_account(user: &mut GooseUser) -> TransactionResult {
    let private_key = PrivateKey::Ed25519(Ed25519PrivateKey::from(Seed::new_random()));
    let public_key = private_key.public_key();
    let account_id = public_key.derive_account_id();

    let sequence = fund_account(&account_id).await;

    let issuance_create = CFTokenIssuanceCreate {
        transaction_type: TransactionType::CFTokenIssuanceCreate,
        transfer_fee: None,
        flags: None,
        sequence,
        last_ledger_sequence: None,
        ticket_sequence: None,
        maximum_amount: u64::MAX,
        fee: XrpAmount::of_drops(1000).unwrap(),
        signing_pub_key: public_key.clone(),
        txn_signature: None,
        account: account_id.clone(),
        asset_scale: 10,
        asset_code: Hash160::try_from_hex(ASSET_CODE).unwrap(),
    };

    let signed_issuance_create = issuance_create.sign_with(&private_key).unwrap();
    let result = RPC_CLIENT.submit(&SubmitRequest {
        tx_blob: hex::encode_upper(signed_issuance_create.to_bytes().unwrap()).as_str()
    }).await.unwrap();

    if result.engine_result != "tesSUCCESS" {
        panic!("got {} when creating issuance.", result.engine_result)
    }

    let mut destinations = Vec::with_capacity(5);
    for _ in 0..destinations.capacity() {
        destinations.push(new_dest(&signed_issuance_create.account, &signed_issuance_create.asset_code).await.account_id)
    }

    let account = Account {
        account_id,
        private_key,
        public_key,
        destinations,
        sequence,
    };

    user.set_session_data(account);
    Ok(())
}

async fn new_dest(issuer: &AccountId, asset_code: &Hash160) -> Account {
    let private_key = PrivateKey::Ed25519(Ed25519PrivateKey::from(Seed::new_random()));
    let public_key = private_key.public_key();
    let account_id = public_key.derive_account_id();
    let sequence = fund_account(&account_id).await;

    let cftoken_create = CFTokenCreate {
        transaction_type: TransactionType::CFTokenCreate,
        flags: None,
        sequence,
        last_ledger_sequence: None,
        ticket_sequence: None,
        fee: XrpAmount::of_drops(1000).unwrap(),
        signing_pub_key: public_key.clone(),
        txn_signature: None,
        account: account_id.clone(),
        issuer: issuer.clone(),
        asset_code: Hash160::try_from_hex(ASSET_CODE).unwrap(),
    };

    let signed_create = cftoken_create.sign_with(&private_key).unwrap();
    let result = RPC_CLIENT.submit(&SubmitRequest {
        tx_blob: hex::encode_upper(signed_create.to_bytes().unwrap()).as_str()
    }).await.unwrap();
    if result.engine_result != "tesSUCCESS" {
        panic!("got {} when creating issuance.", result.engine_result)
    }
    Account {
        private_key,
        public_key,
        destinations: vec![],
        account_id,
        sequence,
    }
}

static MASTER_ACCOUNT: Lazy<Account> = Lazy::new(|| {
    let pk = Secp256k1PrivateKey::from_base58_seed("snoPBrXtMeMyMHUVTgbuqAfg1SUTb").unwrap();
    let pub_k = pk.public_key();
    Account {
        account_id: pub_k.derive_account_id(),
        private_key: PrivateKey::Secp256k1(pk),
        public_key: PublicKey::Secp256k1(pub_k),
        destinations: vec![],
        sequence: 0,
    }
});

pub async fn fund_account(account_id: &AccountId) -> u32 {
    let account_info = RPC_CLIENT
        .account_info(&AccountInfoRequest::new(MASTER_ACCOUNT.account_id.clone()))
        .await.unwrap();

    let mut payment = Payment {
        account: MASTER_ACCOUNT.account_id.clone(),
        destination: account_id.clone(),
        transaction_type: TransactionType::Payment,
        flags: None,
        source_tag: None,
        sequence: account_info.account_data.sequence,
        destination_tag: None,
        last_ledger_sequence: None,
        ticket_sequence: None,
        account_txn_id: None,
        invoice_id: None,
        signers: None,
        memos: None,
        signing_pub_key: MASTER_ACCOUNT.public_key.clone(),
        txn_signature: None,
        paths: None,
        amount: Amount::Xrp(XrpAmount::of_drops(1_000 * ONE_XRP_IN_DROPS).unwrap()),
        fee: XrpAmount::of_drops(1 * ONE_XRP_IN_DROPS).unwrap(),
        send_max: None,
        deliver_min: None,
    };

    let signed_payment = payment
        .sign_with(&MASTER_ACCOUNT.private_key)
        .unwrap();
    let submit_result = RPC_CLIENT
        .submit(&SubmitRequest {
            tx_blob: hex::encode_upper(signed_payment.to_bytes().unwrap()).as_str(),
        }).await
        .unwrap();

    if submit_result.engine_result != "tesSUCCESS" {
        panic!("funding failed! res code: {}", submit_result.engine_result)
    }

    loop {
        if let Ok(account_info) = RPC_CLIENT
            .account_info(&AccountInfoRequest::new(account_id.clone()))
            .await {
            return account_info.account_data.sequence;
        }
    }
}

async fn send_payment(user: &mut GooseUser) -> TransactionResult {
    let user_account = user.get_session_data_unchecked::<Account>();
    let payment = Payment {
        account: user_account.account_id.clone(),
        destination: user_account.destinations.get(user.get_iterations() % user_account.destinations.len()).unwrap().clone(),
        transaction_type: TransactionType::Payment,
        flags: None,
        source_tag: None,
        sequence: user_account.sequence,
        destination_tag: None,
        last_ledger_sequence: None,
        ticket_sequence: None,
        account_txn_id: None,
        invoice_id: None,
        signers: None,
        memos: None,
        signing_pub_key: user_account.public_key.clone(),
        txn_signature: None,
        paths: None,
        amount: Amount::Cft(
            CftAmount::new(
                1,
                Currency::from_str("USD").unwrap(),
                user_account.account_id.clone()
            )
        ),
        fee: XrpAmount::of_drops(12).unwrap(),
        send_max: None,
        deliver_min: None,
    };
    let signed_payment = payment
        .sign_with(&user_account.private_key)
        .unwrap();
    let signed_blob = hex::encode_upper(signed_payment.to_bytes().unwrap());
    let request = JsonRpcRequest::from(SubmitRequest {
        tx_blob: signed_blob.as_str()
    });

    let mut response = user.post_json("/", &request).await?;
    if let Ok(res) = response.response {
        let json_value = res.json::<Value>().await.unwrap();
        if let Some(json_result) = json_value.get("result") {
            if let Some(error) = json_result.get("error") {
                user.set_failure(format!("error submitting {}", error).as_str(), &mut response.request, None, None)?;
            } else if let Some(result_code) = json_result.get("engine_result") {
                let account = user.get_session_data_unchecked_mut::<Account>();
                account.sequence += 1;
                if let Some(res_code_str) = result_code.as_str() {
                    if res_code_str == "tesSUCCESS" {
                        user.set_success(&mut response.request)?;
                    } else {
                        user.set_failure(format!("non tesSUCCESS status {}", res_code_str).as_str(), &mut response.request, None, None)?;
                    }
                }
            }
        }
    }
    Ok(())
}

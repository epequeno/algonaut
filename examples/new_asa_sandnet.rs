use algonaut::core::{Address, MicroAlgos};
use algonaut::transaction::{ConfigureAsset, Txn};
use algonaut::{Algod, Kmd};
use dotenv::dotenv;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    // kmd manages wallets and accounts
    let kmd = Kmd::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .client_v1()?;

    // first we obtain a handle to our wallet
    let list_response = kmd.list_wallets()?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => return Err("Wallet not found".into()),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "")?;
    let wallet_handle_token = init_response.wallet_handle_token;
    println!("Wallet Handle: {}", wallet_handle_token);

    // an account with some funds in our sandbox
    let from_address = Address::from_string(env::var("ACCOUNT")?.as_ref())?;
    println!("Sender: {:?}", from_address);

    let to_address =
        Address::from_string("2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY")?;
    println!("Receiver: {:?}", to_address);

    // algod has a convenient method that retrieves basic information for a transaction
    let algod = Algod::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .client_v2()?;

    let params = algod.transaction_params()?;
    println!("Last round: {}", params.last_round);

    // we are ready to build the transaction
    let t = Txn::new()
        .sender(from_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 1000)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(100_000))
        .asset_configuration(
            ConfigureAsset::new()
                .config_asset(0)
                .total(10)
                .default_frozen(false)
                .unit_name("EIRI".to_owned())
                .asset_name("Naki".to_owned())
                .manager(from_address)
                .reserve(from_address)
                .freeze(from_address)
                .clawback(from_address)
                .url("example.com".to_owned())
                .meta_data_hash(Vec::new())
                .decimals(2)
                .build(),
        )
        .build();

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t)?;

    // broadcast the transaction to the network
    let send_response = algod.broadcast_raw_transaction(&sign_response.signed_transaction)?;

    println!("Transaction ID: {}", send_response.tx_id);

    Ok(())
}
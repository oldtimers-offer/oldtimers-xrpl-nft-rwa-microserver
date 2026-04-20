use anyhow::{anyhow, Result};
use xrpl::core::binarycodec::{encode, encode_for_signing};
use xrpl::core::keypairs::sign as keypairs_sign;
use xrpl::models::transactions::nftoken_mint::{NFTokenMint, NFTokenMintFlag};
use xrpl::models::transactions::CommonFields;
use xrpl::models::transactions::TransactionType;
use xrpl::wallet::Wallet;
use actix_web::rt::time;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MintResult {
    pub nft_id: String,
    pub tx_hash: Option<String>,
    pub uri: String,
}

async fn get_nft_id_from_tx(
    tx_hash: &str,
    node_url: &str,
    expected_uri_hex: &str,
) -> anyhow::Result<String> {
    let body = serde_json::json!({
        "method": "tx",
        "params": [{
            "transaction": tx_hash
        }]
    });

    let resp = reqwest::Client::new()
        .post(node_url)
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let nodes = resp["result"]["meta"]["AffectedNodes"]
        .as_array()
        .ok_or_else(|| anyhow!("No affected nodes"))?;

    for node in nodes {
        if let Some(modified) = node.get("ModifiedNode") {
            if modified["LedgerEntryType"] != "NFTokenPage" {
                continue;
            }

            let final_nfts = modified["FinalFields"]["NFTokens"]
                .as_array()
                .cloned()
                .unwrap_or_default();

            let prev_nfts = modified["PreviousFields"]["NFTokens"]
                .as_array()
                .cloned()
                .unwrap_or_default();

            let prev_ids: HashSet<String> = prev_nfts
                .iter()
                .filter_map(|n| n["NFToken"]["NFTokenID"].as_str().map(|s| s.to_string()))
                .collect();

            for nft in final_nfts {
                let uri = nft["NFToken"]["URI"].as_str();
                let id = nft["NFToken"]["NFTokenID"].as_str();

                if let (Some(uri), Some(id)) = (uri, id) {
                    if uri.eq_ignore_ascii_case(expected_uri_hex) && !prev_ids.contains(id) {
                        return Ok(id.to_string());
                    }
                }
            }
        }

        if let Some(created) = node.get("CreatedNode") {
            if created["LedgerEntryType"] != "NFTokenPage" {
                continue;
            }

            if let Some(nfts) = created["NewFields"]["NFTokens"].as_array() {
                for nft in nfts {
                    let uri = nft["NFToken"]["URI"].as_str();
                    let id = nft["NFToken"]["NFTokenID"].as_str();

                    if let (Some(uri), Some(id)) = (uri, id) {
                        if uri.eq_ignore_ascii_case(expected_uri_hex) {
                            return Ok(id.to_string());
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("NFT ID not found in transaction"))
}

pub struct VehiclePassportMinter;

impl VehiclePassportMinter {
    pub fn new() -> Self {
        Self
    }

    pub async fn mint_from_uri(&self, uri: &str, seed: &str, node_url: &str) -> Result<MintResult> {
        let wallet = Wallet::new(seed, 0)
            .map_err(|e| anyhow!("Wallet error: {:?}", e))?;

        let uri_hex = hex::encode(uri.as_bytes()).to_uppercase();

        let sequence = self.get_sequence(&wallet.classic_address, node_url).await?;
        let last_ledger = self.get_latest_ledger(node_url).await? + 20;

        let mut nft_mint = NFTokenMint {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::NFTokenMint,
                fee: Some("12".into()),
                sequence: Some(sequence),
                last_ledger_sequence: Some(last_ledger),
                signing_pub_key: Some(wallet.public_key.clone().into()),
                ..Default::default()
            },
            nftoken_taxon: 1,
            ..Default::default()
        };

        nft_mint = nft_mint
            .with_flag(NFTokenMintFlag::TfTransferable)
            .with_uri(uri_hex.into());

        let bytes_for_signing = encode_for_signing(&nft_mint)
            .map_err(|e| anyhow!("encode_for_signing: {:?}", e))?;

        let raw_bytes = hex::decode(&bytes_for_signing)
            .map_err(|e| anyhow!("Hex decode: {}", e))?;

        let signature = keypairs_sign(&raw_bytes, &wallet.private_key)
            .map_err(|e| anyhow!("Sign: {:?}", e))?;

        nft_mint.common_fields.txn_signature = Some(signature.into());

        let tx_blob = encode(&nft_mint)
            .map_err(|e| anyhow!("Final encode: {:?}", e))?;

        let response = self.submit_blob(&tx_blob, node_url).await?;

        let engine_result = response["result"]["engine_result"]
            .as_str()
            .unwrap_or("UNKNOWN");

        if engine_result != "tesSUCCESS" {
            return Err(anyhow!(
                "Transaction failed: {} — {}",
                engine_result,
                response["result"]["engine_result_message"]
                    .as_str()
                    .unwrap_or("")
            ));
        }

        let tx_hash = response["result"]["tx_json"]["hash"]
            .as_str()
            .map(|s: &str| s.to_string());

        let tx_hash_str = tx_hash
            .as_ref()
            .ok_or_else(|| anyhow!("Missing tx hash"))?;

        // ⏳ čekanje + retry
        let mut attempts = 0;
        let max_attempts = 10;

        let uri_hex = hex::encode(uri.as_bytes()).to_uppercase();

        let token_id = loop {
            match get_nft_id_from_tx(tx_hash_str, node_url, &uri_hex).await {
                Ok(id) => break id,
                Err(_) => {
                    if attempts >= max_attempts {
                        return Err(anyhow!("NFT ID not found after retries"));
                    }

                    attempts += 1;

                    println!("⏳ Waiting for XRPL validation... attempt {}", attempts);

                    time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        };

        Ok(MintResult {
            nft_id: token_id,
            tx_hash,
            uri: uri.to_string(),
        })
    }

    async fn get_sequence(&self, address: &str, node_url: &str) -> Result<u32> {
        let body = serde_json::json!({
            "method": "account_info",
            "params": [{ "account": address, "ledger_index": "current" }]
        });

        let resp = reqwest::Client::new()
            .post(node_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP: {}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("JSON: {}", e))?;

        Ok(resp["result"]["account_data"]["Sequence"]
            .as_u64()
            .ok_or_else(|| anyhow!("Sequence not found: {}", resp))? as u32)
    }

    async fn get_latest_ledger(&self, node_url: &str) -> Result<u32> {
        let body = serde_json::json!({
            "method": "ledger",
            "params": [{ "ledger_index": "validated" }]
        });

        let resp = reqwest::Client::new()
            .post(node_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP: {}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("JSON: {}", e))?;

        Ok(resp["result"]["ledger_index"]
            .as_u64()
            .ok_or_else(|| anyhow!("Ledger index not found"))? as u32)
    }

    async fn submit_blob(&self, tx_blob: &str, node_url: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "method": "submit",
            "params": [{ "tx_blob": tx_blob }]
        });

        Ok(reqwest::Client::new()
            .post(node_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP: {}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("JSON: {}", e))?)
    }
}
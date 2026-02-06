//! Pagination integration tests.

mod helpers;

use helpers::{client_config_with_mock, load_fixture, start_mock_server};
use morpho_rs_api::{
    ClientConfig, NamedChain, VaultFiltersV1, VaultFiltersV2, VaultQueryOptionsV1,
    VaultQueryOptionsV2, VaultV1Client, VaultV2Client,
};
use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create a V1 page response JSON with specified vault count and total.
fn v1_page_fixture(vaults: &[(&str, &str, &str)], count_total: i64) -> String {
    let items: Vec<String> = vaults
        .iter()
        .map(|(addr, name, symbol)| {
            format!(
                r#"{{
                    "id": "vault-{symbol}",
                    "address": "{addr}",
                    "name": "{name}",
                    "symbol": "{symbol}",
                    "chain": {{ "id": 1, "network": "ETHEREUM" }},
                    "listed": true,
                    "featured": false,
                    "whitelisted": true,
                    "asset": {{
                        "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                        "symbol": "USDC",
                        "name": "USD Coin",
                        "decimals": 6,
                        "priceUsd": 1.0
                    }},
                    "state": {{
                        "curator": "0x1234567890123456789012345678901234567890",
                        "owner": "0x2345678901234567890123456789012345678901",
                        "guardian": "0x3456789012345678901234567890123456789012",
                        "totalAssets": "1000000000000",
                        "totalAssetsUsd": 1000000.0,
                        "totalSupply": "1000000000000",
                        "fee": 0.1,
                        "timelock": "86400",
                        "apy": 0.05,
                        "netApy": 0.045,
                        "sharePrice": "1000000",
                        "allocation": []
                    }},
                    "allocators": [],
                    "warnings": []
                }}"#,
            )
        })
        .collect();

    let count = vaults.len();
    format!(
        r#"{{"data":{{"vaults":{{"items":[{items}],"pageInfo":{{"count":{count},"countTotal":{count_total}}}}}}}}}"#,
        items = items.join(","),
    )
}

/// Create a V2 page response JSON with specified vault count and total.
fn v2_page_fixture(vaults: &[(&str, &str, &str)], count_total: i64) -> String {
    let items: Vec<String> = vaults
        .iter()
        .map(|(addr, name, symbol)| {
            format!(
                r#"{{
                    "id": "vault-{symbol}",
                    "address": "{addr}",
                    "name": "{name}",
                    "symbol": "{symbol}",
                    "chain": {{ "id": 1, "network": "ETHEREUM" }},
                    "listed": true,
                    "whitelisted": true,
                    "asset": {{
                        "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                        "symbol": "USDC",
                        "name": "USD Coin",
                        "decimals": 6,
                        "priceUsd": 1.0
                    }},
                    "curator": {{ "address": "0x1234567890123456789012345678901234567890" }},
                    "owner": {{ "address": "0x2345678901234567890123456789012345678901" }},
                    "totalAssets": "1000000000000",
                    "totalAssetsUsd": 1000000.0,
                    "totalSupply": "1000000000000",
                    "sharePrice": 1.0,
                    "performanceFee": 0.1,
                    "managementFee": 0.02,
                    "avgApy": 0.06,
                    "avgNetApy": 0.052,
                    "apy": 0.065,
                    "netApy": 0.057,
                    "liquidity": "500000000000",
                    "liquidityUsd": 500000.0,
                    "adapters": {{ "items": [] }},
                    "rewards": [],
                    "warnings": []
                }}"#,
            )
        })
        .collect();

    let count = vaults.len();
    format!(
        r#"{{"data":{{"vaultV2s":{{"items":[{items}],"pageInfo":{{"count":{count},"countTotal":{count_total}}}}}}}}}"#,
        items = items.join(","),
    )
}

#[tokio::test]
async fn test_v1_pagination_multiple_pages() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let server = start_mock_server().await;

    let page1 = v1_page_fixture(
        &[
            ("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "Vault A", "vA"),
            ("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Vault B", "vB"),
        ],
        4,
    );
    let page2 = v1_page_fixture(
        &[
            ("0xcccccccccccccccccccccccccccccccccccccccc", "Vault C", "vC"),
            ("0xdddddddddddddddddddddddddddddddddddddddd", "Vault D", "vD"),
        ],
        4,
    );

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    Mock::given(method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            let call_num = counter_clone.fetch_add(1, Ordering::SeqCst);
            if call_num == 0 {
                ResponseTemplate::new(200).set_body_string(page1.clone())
            } else {
                ResponseTemplate::new(200).set_body_string(page2.clone())
            }
        })
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server).with_page_size(2);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 4);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert_eq!(vaults[0].name, "Vault A");
    assert_eq!(vaults[3].name, "Vault D");
}

#[tokio::test]
async fn test_v1_pagination_with_limit_stops_early() {
    let server = start_mock_server().await;

    // Page 1: 2 vaults, 4 total
    let page1 = v1_page_fixture(
        &[
            ("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "Vault A", "vA"),
            ("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Vault B", "vB"),
        ],
        4,
    );

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .expect(1) // Only one request should be made
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server).with_page_size(2);
    let client = VaultV1Client::with_config(config);

    let options = VaultQueryOptionsV1::new().limit(2);
    let vaults = client.get_vaults_with_options(options).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_v1_pagination_empty_second_page() {
    let server = start_mock_server().await;

    // Page 1: 2 vaults, 2 total (no second page needed)
    let page1 = v1_page_fixture(
        &[
            ("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "Vault A", "vA"),
            ("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Vault B", "vB"),
        ],
        2,
    );

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .expect(1) // Should stop after first page
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server).with_page_size(2);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_v2_pagination_multiple_pages() {
    let server = start_mock_server().await;

    let page1 = v2_page_fixture(
        &[
            ("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "Vault A", "vA"),
            ("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Vault B", "vB"),
        ],
        3,
    );
    let page2 = v2_page_fixture(
        &[("0xcccccccccccccccccccccccccccccccccccccccc", "Vault C", "vC")],
        3,
    );

    Mock::given(method("POST"))
        .and(body_string_contains(r#""skip":0"#))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(body_string_contains(r#""skip":2"#))
        .respond_with(ResponseTemplate::new(200).set_body_string(page2))
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server).with_page_size(2);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 3);
}

#[tokio::test]
async fn test_v2_pagination_with_limit() {
    let server = start_mock_server().await;

    let page1 = v2_page_fixture(
        &[
            ("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "Vault A", "vA"),
            ("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Vault B", "vB"),
        ],
        4,
    );

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(page1))
        .expect(1)
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server).with_page_size(2);
    let client = VaultV2Client::with_config(config);

    let options = VaultQueryOptionsV2::new().limit(2);
    let vaults = client.get_vaults_with_options(options).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_v1_single_page_fits_all() {
    let server = start_mock_server().await;

    // All results fit in one page
    let body = load_fixture("v1_list");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .expect(1) // Only one request
        .mount(&server)
        .await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

//! GraphQL scalar type conversions to Rust/alloy types.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

/// GraphQL Address scalar type (represented as String in GraphQL).
pub type GqlAddress = String;

/// GraphQL BigInt scalar type (represented as String in GraphQL).
pub type GqlBigInt = String;

/// Parse a GraphQL Address string into an alloy Address.
pub fn parse_address(s: &str) -> Option<Address> {
    Address::from_str(s).ok()
}

/// Parse a GraphQL BigInt string into a U256.
pub fn parse_bigint(s: &str) -> Option<U256> {
    U256::from_str(s).ok()
}

/// Deserialize an optional address from GraphQL response.
pub fn deserialize_optional_address<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.and_then(|s| parse_address(&s)))
}

/// Deserialize an address from GraphQL response.
pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    parse_address(&s).ok_or_else(|| serde::de::Error::custom(format!("Invalid address: {}", s)))
}

/// Deserialize an optional BigInt from GraphQL response into U256.
pub fn deserialize_optional_bigint<'de, D>(deserializer: D) -> Result<Option<U256>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.and_then(|s| parse_bigint(&s)))
}

/// Deserialize a BigInt from GraphQL response into U256.
pub fn deserialize_bigint<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    parse_bigint(&s).ok_or_else(|| serde::de::Error::custom(format!("Invalid BigInt: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x1234567890123456789012345678901234567890");
        assert!(addr.is_some());

        let invalid = parse_address("not-an-address");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_parse_bigint() {
        let val = parse_bigint("1000000000000000000");
        assert!(val.is_some());
        assert_eq!(val.unwrap(), U256::from(1_000_000_000_000_000_000u64));

        let invalid = parse_bigint("not-a-number");
        assert!(invalid.is_none());
    }
}

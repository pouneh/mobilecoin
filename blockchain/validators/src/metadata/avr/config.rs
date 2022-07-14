// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Configuration for the avr history bootstrap file.

use crate::error::ParseError;

use mc_blockchain_types::{BlockIndex, VerificationReport, VerificationSignature};
use mc_common::ResponderId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{hex, serde_as, DeserializeAs, SerializeAs};
use std::{fs, option::Option, path::Path};

/// Struct for reading historical Intel Attestation Verification Report
/// (AVR) data from a configuration file.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AvrHistoryConfig {
    // List of AvrHistoryRecord objects sorted by ResponderId and block range
    pub node: Vec<AvrHistoryRecord>,
}

/// Stores a historical AVR record (or lack thereof) for a given
/// [ResponderId] and block range
#[serde_as]
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AvrHistoryRecord {
    /// Uri of the consensus node
    pub responder_id: ResponderId,

    /// Block the AVR Report for the signing key becomes valid
    pub first_block_index: BlockIndex,

    /// Final block the AVR Report for the signing key is valid
    pub last_block_index: BlockIndex,

    /// AVR Report (or lack thereof) for the node & block ranges
    #[serde_as(as = "Option<VerificationReportShadow>")]
    #[serde(default)]
    pub avr: Option<VerificationReport>,
}

impl AvrHistoryConfig {
    /// Load the [AvrHistoryConfig] from a .json or .toml file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file containing
    /// the history of AVRs generated MobileCoin consensus node
    /// enclaves
    pub fn try_from_file(path: impl AsRef<Path>) -> Result<AvrHistoryConfig, ParseError> {
        let data = fs::read_to_string(path)?;

        if let Ok(mut config) = serde_json::from_str(&data): Result<AvrHistoryConfig, _> {
            config.node.sort();
            Ok(config)
        } else if let Ok(mut config) = toml::from_str(&data): Result<AvrHistoryConfig, _> {
            config.node.sort();
            Ok(config)
        } else {
            Err(ParseError::UnsuportedFileFormat)
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize)]
#[serde(remote = "VerificationReport")]
/// Struct to shadow the mc_blockchain_types's VerificationReport for
/// serialization purposes
pub struct VerificationReportShadow {
    /// Report Signature bytes, from the X-IASReport-Signature HTTP header.
    #[serde_as(as = "hex::Hex")]
    pub sig: VerificationSignature,

    /// Attestation Report Signing Certificate Chain, as an array of
    /// DER-formatted bytes, from the X-IASReport-Signing-Certificate HTTP
    /// header.
    #[serde_as(as = "Vec<hex::Hex>")]
    pub chain: Vec<Vec<u8>>,

    /// The raw report body JSON, as a byte sequence
    pub http_body: String,
}

// SerializeAs and Deserialize are needed to get VerificationReportShadow (serde
// remote) to work with container types (ie. Option<VerificationReport> )
impl SerializeAs<VerificationReport> for VerificationReportShadow {
    fn serialize_as<S>(source: &VerificationReport, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        VerificationReportShadow::serialize(source, serializer)
    }
}

impl<'de> DeserializeAs<'de, VerificationReport> for VerificationReportShadow {
    fn deserialize_as<D>(deserializer: D) -> Result<VerificationReport, D::Error>
    where
        D: Deserializer<'de>,
    {
        VerificationReportShadow::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use serde_json;
    use super::*;
    use crate::test_utils::{
        get_avr_history_config, SAMPLE_AVR_HISTORY_JSON, get_ias_reports,
        SAMPLE_AVR_HISTORY_TOML,
    };
    use tempfile::TempDir;

    #[test]
    fn test_avr_history_serialization_roundtrip_works() {
        // Get a manually constructed AVR history config to act as control data

        let control_avr_history = get_avr_history_config();

        // Serialize the config to JSON & TOML
        let toml_str = toml::to_string_pretty(&control_avr_history).unwrap();
        let json_str = serde_json::to_string_pretty(&control_avr_history).unwrap();

        let history_from_toml: AvrHistoryConfig = toml::from_str(toml_str.as_str()).unwrap();
        let history_from_json: AvrHistoryConfig = serde_json::from_str(json_str.as_str()).unwrap();

        // Assert that deserialization from JSON and TOML is the same as the original
        // config
        assert_eq!(control_avr_history, history_from_toml);
        assert_eq!(control_avr_history, history_from_json);
    }

    #[test]
    fn test_avr_history_load_from_disk() {
        // Get a manually constructed AVR history config to act as control data
        let control_avr_history = get_avr_history_config();

        // Write JSON and TOML to disk
        let temp = TempDir::new().unwrap();
        let path_json = temp.path().join("avr-history.json");
        let path_toml = temp.path().join("avr-history.toml");
        fs::write(&path_json, SAMPLE_AVR_HISTORY_JSON).unwrap();
        fs::write(&path_toml, SAMPLE_AVR_HISTORY_TOML).unwrap();

        // Load the config from disk
        let avr_history_from_json = AvrHistoryConfig::try_from_file(path_json).unwrap();
        let avr_history_from_toml = AvrHistoryConfig::try_from_file(path_toml).unwrap();

        // Check that the avr histories loaded from disk are the same as the control
        assert_eq!(control_avr_history, avr_history_from_json);
        assert_eq!(control_avr_history, avr_history_from_toml);
    }
}
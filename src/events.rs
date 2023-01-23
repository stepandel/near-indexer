use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "standard")]
#[serde(rename_all = "snake_case")]
pub(crate) enum NearEvent {
    Nep141(Nep141Event),
    Nep171(Nep171Event),
}

// *** NEP-141 FT ***
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Nep141Event {
    pub version: String,
    #[serde(flatten)]
    pub event_kind: Nep141EventKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Nep141EventKind {
    FtMint(Vec<FtMintData>),
    FtTransfer(Vec<FtTransferData>),
    FtBurn(Vec<FtBurnData>),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FtMintData {
    pub owner_id: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FtTransferData {
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FtBurnData {
    pub owner_id: String,
    pub amount: String,
    pub memo: Option<String>,
}

// *** NEP-171 NFT ***
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Nep171Event {
    pub version: String,
    #[serde(flatten)]
    pub event_kind: Nep171EventKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Nep171EventKind {
    NftMint(Vec<NftMintData>),
    NftTransfer(Vec<NftTransferData>),
    NftBurn(Vec<NftBurnData>),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct NftMintData {
    pub owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct NftTransferData {
    pub authorized_id: Option<String>,
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct NftBurnData {
    pub authorized_id: Option<String>,
    pub owner_id: String,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,
}

pub(crate) fn extract_events(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
) -> Vec<NearEvent> {
    let prefix = "EVENT_JSON:";
    outcome.execution_outcome.outcome.logs.iter().filter_map(|untrimmed_log| {
        let log = untrimmed_log.trim();
        if !log.starts_with(prefix) {
            return None;
        }

        match serde_json::from_str::<'_, NearEvent>(
            log[prefix.len()..].trim(),
        ) {
            Ok(result) => Some(result),
            Err(err) => {
                info!(
                    target: "indexer_example",
                    "Provided event log does not correspond to any of formats defined in NEP. Will ignore this event. \n {:#?} \n{:#?}",
                    err,
                    untrimmed_log,
                );
                None
            }
        }
    }).collect()
}
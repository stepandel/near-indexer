use tracing::info;
use std::collections::HashSet;

use near_indexer::near_primitives::views;
use near_indexer::near_primitives::types::AccountId;

pub(crate) fn process_receipts(receipts: &[views::ReceiptView]) {

    let nft_contract_id: AccountId = AccountId::new_unvalidated(String::from("nftsmartcontract.test.near"));
    let watched_addresses: HashSet<AccountId> = HashSet::from([nft_contract_id]);

    for receipt in receipts {
        // Filter by receiver_id - contract_id

        let receiver_id = &receipt.receiver_id;

        assert_eq!(
            watched_addresses.contains(&*receiver_id),
            true,
            "Not watching this address!"
        );

        info!(
            target: "indexer_example",
            "Receipt args: {:#?}",
            &receipt,
        );

        let receipt = &receipt.receipt;
        match receipt {
            views::ReceiptEnumView::Action {
                actions,
                ..
            } => {
                let actions_iter = actions.iter();
                for action in actions_iter {
                    let (_action_kind, args) =
                    crate::serializers::extract_action_type_and_value_from_action_view(action);
                    info!(
                        target: "indexer_example",
                        "Action args: {:#?}",
                        &args,
                    );
                }
            }
            _ => (),
        }
    }
}


// TODO: - Get Function Args from Receipt

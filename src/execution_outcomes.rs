use tracing::{ info, error, warn };
use near_indexer::IndexerExecutionOutcomeWithReceipt;
use near_indexer::near_primitives::{ types::AccountId, views::ExecutionStatusView, views::ReceiptEnumView };
use std::collections::HashSet;
use crate::functions;
use crate::models::token;
use crate::events;

pub(crate) async fn process_execution_outcomes(
    pool: &mongodb::Client,
    execution_outcomes: &[IndexerExecutionOutcomeWithReceipt],
) {
    
    let nft_contract_id: AccountId = AccountId::try_from(crate::NFT_CONTRACT_ID.to_string()).unwrap();
    let ft_contract_id: AccountId = AccountId::try_from(crate::FT_CONTRACT_ID.to_string()).unwrap();
    let watched_addresses: HashSet<AccountId> = HashSet::from([nft_contract_id,ft_contract_id]);

    for execution_outcome in execution_outcomes {
    
        let executor_id = &execution_outcome.execution_outcome.outcome.executor_id;

        // Check 1: - contract_id
        if watched_addresses.contains(&*executor_id) {

            info!(
                target: crate::INDEXER,
                "Execution outcome: {:#?}",
                &execution_outcome,
            );

            let receiver_id = &execution_outcome.receipt.receiver_id;
            let success = &execution_outcome.execution_outcome.outcome.status;
            // Check 2: - SuccessValue
            match success {
                ExecutionStatusView::SuccessValue(_) => {
                    
                    // Check 3: - FunctionCall
                    let receipt = &execution_outcome.receipt.receipt;

                    match receipt {
                        ReceiptEnumView::Action {
                            actions,
                            ..
                        } => {
                            for action in actions {
                                
                                if let Some(args) = functions::get_arg_from_function_call(&action) {

                                    info!(
                                        target: crate::INDEXER,
                                        "Args serialized: {:#?}",
                                        &args,
                                    );

                                    let events = events::extract_events(&execution_outcome);

                                    token::process_token_event(pool, receiver_id, &args, &events).await;
                                }
                            }
                        },
                        _ => (),
                    }
                },
                _ => {
                    error!(
                        target: crate::INDEXER,
                        "Function call Failed!",
                    )
                },
            }
        } else {
            warn!(
                target: crate::INDEXER,
                "Not watching these executions",
            )
        }
    }
}
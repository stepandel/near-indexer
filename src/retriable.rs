#[macro_export]
macro_rules! await_retry_or_panic {
    ($query: expr, $number_of_retries: expr, $error_message: expr, $debug_structs: expr $(,)?) => {
        {
            let mut interval = crate::INTERVAL;
            let mut retry_attempt = 0usize;
            loop {
                if retry_attempt == $number_of_retries {
                    return Err(
                        anyhow::anyhow!(
                            "Failed to perform request after {} attempts. Stop trying.",
                            $number_of_retries
                        )
                    );
                }
                retry_attempt += 1;

                match $query.await {
                    Ok(res) => {

                        tracing::info!(
                            target: crate::INDEXER,
                            "Execution succeeded: {:#?}",
                            &res,
                        );

                        break Some(res);
                    },
                    Err(async_error) => {

                        tracing::error!(
                            target: crate::INDEXER,
                            "Error occurred during {}: \n{:#?} \n{:#?} \n Retrying in {} milliseconds...",
                            async_error,
                            &$error_message,
                            &$debug_structs,
                            interval.as_millis(),
                        );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };
}

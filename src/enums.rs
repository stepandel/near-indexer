#[derive(Debug, Clone)]
pub enum ActionKind {
    CreateAccount,
    DeployContract,
    FunctionCall,
    Transfer,
    Stake,
    AddKey,
    DeleteKey,
    DeleteAccount,
}

#[derive(Debug, Clone)]
pub enum ExecutionOutcomeStatus {
    Unknown,
    Failure,
    SuccessValue,
    SuccessReceiptId,
}
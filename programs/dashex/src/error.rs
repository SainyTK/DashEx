use anchor_lang::prelude::*;

#[error_code]
pub enum DashExError {
    #[msg("The supplied amount is invalid.")]
    InvalidAmount,
    #[msg("The AMM reserve would be invalid.")]
    InvalidReserve,
    #[msg("Arithmetic overflow.")]
    MathOverflow,
    #[msg("The market is not available for this trade.")]
    MarketUnavailable,
    #[msg("The order would exceed its limit price.")]
    LimitPriceExceeded,
    #[msg("The account does not have enough free collateral.")]
    InsufficientFreeCollateral,
    #[msg("The requested close amount exceeds the position.")]
    CloseAmountExceedsPosition,
    #[msg("The supplied market configuration is invalid.")]
    InvalidMarketConfiguration,
    #[msg("The user account belongs to another wallet.")]
    InvalidUserAccountOwner,
    #[msg("Only the protocol administrator can perform this action.")]
    Unauthorized,
}

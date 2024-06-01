use anchor_lang::prelude::*;

#[error_code]
pub enum AppError {
  #[msg("Tokens are already staked")]
  IsStaked,

  #[msg("Tokens are not staked")]
  NotStaked,

  #[msg("No Token to stake")]
  NoToken,

  #[msg("Over your stake balance")]
  OverStakeBalance,
}

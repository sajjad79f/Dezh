use thiserror::Error;

#[derive(Debug, Error)]
pub enum FirewallError {
    #[error("'nft' was not found -- this adapter targets nftables on Debian; is it installed?")]
    NftNotFound,

    #[error("nft command failed: {0}")]
    CommandFailed(String),
}
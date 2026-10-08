use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionLockOptions {}

#[derive(Debug, Error)]
#[error("Compositor doesn't support ext_session_lock_v1")]
pub struct SessionLockNotSupportedError;

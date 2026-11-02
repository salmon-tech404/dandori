//! Kho khóa mã hóa và quản lý bí mật của Dandori (01_KIEN_TRUC.md mục 6.2 & T08).

#![warn(missing_docs)]

pub mod default_store;
pub mod error;
pub mod keystore;
pub mod secret;
pub mod storage;
pub mod vault;

pub use default_store::{default_key_store, default_key_store_kind, set_default_key_store};
pub use error::VaultError;
pub use keystore::{MasterKeyStore, MemoryKeyStore, OsKeyStore};
pub use secret::SecretBox;
pub use storage::{MemorySecretStorage, SecretStorage, SqliteSecretStorage};
pub use vault::Vault;

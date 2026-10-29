pub mod backup;
pub mod error;
pub mod layout;
pub mod manifest;

pub use backup::{copy_dir_all, safe_backup};
pub use error::{Result, RpgError};
pub use layout::{detect_game_layout, GameLayout};
pub use manifest::{read_system_info, update_system_encryption_flags, SystemInfo};

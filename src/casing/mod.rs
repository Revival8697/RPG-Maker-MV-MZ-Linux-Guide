pub mod parser;
pub mod resolver;
pub mod transaction;

pub use parser::extract_all_references;
pub use resolver::select_canonical_stem;
pub use transaction::{execute_fix_casing, resolve_ci_dir, CasingReport, VALID_EXTS};

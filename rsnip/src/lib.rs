pub mod application;
pub mod cli;
pub mod complete;
pub mod config;
pub mod domain;
pub mod fuzzy;
pub mod infrastructure;
pub mod path_utils;
pub mod template;
pub mod util;

#[cfg(test)]
/// must be public to be used from integration tests
mod tests {
    use crate::util::testing;
    use super::*;
    #[ctor::ctor]
    fn init() {
        testing::init_test_setup().expect("Failed to initialize test setup");
    }
}


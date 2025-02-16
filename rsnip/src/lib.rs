pub mod application;
pub mod cli;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod util;

#[cfg(test)]
/// must be public to be used from integration tests
mod tests {
    use crate::util::testing;
    #[ctor::ctor]
    fn init() {
        testing::init_test_setup().expect("Failed to initialize test setup");
    }
}


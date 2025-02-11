//! Integration tests for the library

mod application;
mod infrastructure;

use rsnip::util::testing;

#[ctor::ctor]
fn init() {
    testing::init_test_setup().expect("Failed to initialize test setup");
}

#[cfg(test)]
mod tests {
    use super::*;
}

pub mod leak_test;
pub mod resolved;

pub use leak_test::verify_secure_dns;
pub use resolved::{configure_link_dns, revert_link_dns};

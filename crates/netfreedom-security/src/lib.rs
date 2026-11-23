pub mod nftables;

pub use nftables::{
    apply_killswitch_rules, generate_nftables_ruleset, is_killswitch_active, remove_killswitch_rules,
};

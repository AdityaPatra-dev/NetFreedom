pub mod nftables;
pub mod windows_wfp;

pub use nftables::{
    apply_killswitch_rules, generate_nftables_ruleset, is_killswitch_active, remove_killswitch_rules,
};
pub use windows_wfp::{
    apply_windows_wfp_killswitch, generate_windows_firewall_cleanup_script,
    generate_windows_firewall_script, remove_windows_wfp_killswitch,
};

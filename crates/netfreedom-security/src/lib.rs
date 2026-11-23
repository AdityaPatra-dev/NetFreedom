pub mod leak_test;
pub mod nftables;
pub mod windows_wfp;

pub use leak_test::{
    audit_nftables_ruleset, run_synthetic_leak_tests, verify_double_prefix_trick, KillswitchAudit,
    LeakAuditVerdict,
};
pub use nftables::{
    apply_killswitch_rules, generate_nftables_ruleset, is_killswitch_active, remove_killswitch_rules,
};
pub use windows_wfp::{
    apply_windows_wfp_killswitch, generate_windows_firewall_cleanup_script,
    generate_windows_firewall_script, remove_windows_wfp_killswitch,
};

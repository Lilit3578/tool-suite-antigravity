pub mod macos;

pub use macos::{
    auto_paste_flow,
    check_accessibility_permissions,
    get_active_app,
    restore_focus,
    simulate_cmd_c,
    simulate_cmd_v,
};

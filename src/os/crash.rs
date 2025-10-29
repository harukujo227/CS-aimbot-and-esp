use std::{
    collections::HashMap,
    panic::{self, PanicHookInfo},
    process::Command,
};

pub fn install_crash_handler() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_hook_info| {
        crash_handler(panic_hook_info);
        default_hook(panic_hook_info);
    }));
}

fn crash_handler(_panic_hook_info: &PanicHookInfo) {
    let machine_id = std::fs::read_to_string("/etc/machine-id").unwrap_or(UNKNOWN.to_string());
    let hwid = machine_id.trim_end();
    let mut stacktrace = _panic_hook_info
        .payload()
        .downcast_ref::<&str>()
        .unwrap_or(&"explicit panic")
        .to_string();
    stacktrace.push('\n');
    stacktrace.push_str(&std::backtrace::Backtrace::force_capture().to_string());

    let json = serde_json::json!({"hwid": hwid,"stacktrace": stacktrace});

    let _ = ureq::post("https://deadlocked.holyhades64.workers.dev/stacktrace").send_json(json);

    log::error!("crash reported");
}

const UNKNOWN: &str = "unknown";

pub fn info() {
    let hwid = std::fs::read_to_string("/etc/machine-id")
        .unwrap_or(UNKNOWN.to_owned())
        .trim()
        .to_owned();
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or(UNKNOWN.to_owned())
        .trim()
        .to_owned();
    let distro = distro();
    let desktop = desktop();
    let rust_version = rust_version();
    let git_commit = git_commit();

    let json = serde_json::json!({
        "hwid": hwid,
        "kernel": kernel,
        "distro": distro,
        "desktop": desktop,
        "rust_version": rust_version,
        "git_commit": git_commit
    });
    let _ = ureq::post("https://deadlocked.holyhades64.workers.dev/misc").send_json(json);
}

fn distro() -> String {
    let Ok(content) = std::fs::read_to_string("/etc/os-release") else {
        return UNKNOWN.to_owned();
    };
    let mut info = HashMap::new();
    for line in content.lines() {
        let mut parts = line.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            info.insert(key, value.trim_matches('"'));
        }
    }
    if let Some(name) = info.get("PRETTY_NAME") {
        return (*name).to_owned();
    } else if let Some(name) = info.get("NAME") {
        return (*name).to_owned();
    }
    UNKNOWN.to_owned()
}

fn desktop() -> String {
    let de_vars = [
        "XDG_CURRENT_DESKTOP",
        "DESKTOP_SESSION",
        "GDMSESSION",
        "GNOME_DESKTOP_SESSION_ID",
    ];

    for var in &de_vars {
        if let Ok(val) = std::env::var(var)
            && !val.is_empty()
        {
            return val;
        }
    }

    UNKNOWN.to_string()
}

fn rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or(UNKNOWN.to_owned())
}

fn git_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or(UNKNOWN.to_owned())
}

//! macOS privacy permissions (TCC) for Qwerkeys.
//!
//! The app needs two macOS privacy services:
//!
//! 1. **Input Monitoring** (`kTCCServiceListenEvent`) — required. Reads raw
//!    keystrokes via `IOHIDManager`, which is the whole point of the app.
//! 2. **Accessibility** (`kTCCServiceAccessibility`) — recommended. Allows the
//!    Quartz `CGEventTap` to filter (swallow) the mapped keystrokes so typing
//!    stays quiet while playing.
//!
//! Checks use public, non-interactive preflight APIs so the onboarding page
//! can poll authorization state live (`CGPreflightListenEventAccess` /
//! `AXIsProcessTrusted`). Explicit `request` calls trigger the TCC consent
//! alert, and System Settings panes can be opened directly for manual grants.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    /// `AXIsProcessTrusted()` — `true` when the app holds Accessibility.
    fn AXIsProcessTrusted() -> u8;
    /// `AXIsProcessTrustedWithOptions()` — same check, optionally shows the
    /// system consent alert first. `options` is a CFDictionaryRef whose key is
    /// `kAXTrustedCheckOptionPrompt` mapped to a CFBoolean.
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> u8;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    /// `CGPreflightListenEventAccess()` — silent check for Input Monitoring.
    fn CGPreflightListenEventAccess() -> u8;
    /// `CGRequestListenEventAccess()` — shows the Input Monitoring consent
    /// alert on first use, then returns the current authorization state.
    fn CGRequestListenEventAccess() -> u8;
}

/// `kAXTrustedCheckOptionPrompt` string constant (HIServices).
const K_AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";

/// Bundle identifier for Qwerkeys.
pub const BUNDLE_ID: &str = "com.notenderdreams.qwerkeys";

/// Reset macOS TCC privacy grants for Qwerkeys (clears stale permissions
/// left by older versions or mismatched ad-hoc signatures).
pub fn reset_all() {
    let _ = std::process::Command::new("/usr/bin/tccutil")
        .args(["reset", "ListenEvent", BUNDLE_ID])
        .status();
    let _ = std::process::Command::new("/usr/bin/tccutil")
        .args(["reset", "Accessibility", BUNDLE_ID])
        .status();
}

/// A macOS privacy service required (or recommended) by Qwerkeys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Reads raw keystrokes from connected keyboards (IOHIDManager).
    InputMonitoring,
    /// Allows filtering (suppressing) keystrokes via the CGEventTap.
    Accessibility,
}

impl Permission {
    /// Silent, non-interactive check of the current authorization state.
    /// Safe to call every frame; never prompts.
    pub fn is_granted(self) -> bool {
        match self {
            Self::InputMonitoring => input_monitoring_granted(),
            Self::Accessibility => accessibility_granted(),
        }
    }

    /// Prompt the user for this permission. Returns `true` if authorization
    /// is held when the call completes (the user may still be acting on the
    /// TCC alert, so re-check [`Permission::is_granted`] afterwards).
    pub fn request(self) -> bool {
        if self.is_granted() {
            return true;
        }
        match self {
            Self::InputMonitoring => unsafe { CGRequestListenEventAccess() != 0 },
            Self::Accessibility => unsafe {
                let options = CFDictionary::from_CFType_pairs(&[(
                    CFString::new(K_AX_TRUSTED_CHECK_OPTION_PROMPT),
                    CFBoolean::true_value(),
                )]);
                AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as *const c_void) != 0
            },
        }
    }

    /// `x-apple.systempreferences:` deep link into the matching System
    /// Settings privacy pane.
    pub fn settings_url(self) -> &'static str {
        match self {
            Self::InputMonitoring => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
            }
            Self::Accessibility => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
        }
    }

    /// Open the matching "Privacy & Security" pane in System Settings.
    pub fn open_system_settings(self) {
        let _ = std::process::Command::new("open")
            .arg(self.settings_url())
            .spawn();
    }
}

/// Does the app currently hold the Accessibility permission?
pub fn accessibility_granted() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

/// Does the app currently hold the Input Monitoring permission?
pub fn input_monitoring_granted() -> bool {
    unsafe { CGPreflightListenEventAccess() != 0 }
}

/// Determine whether onboarding should be considered done and dismissed.
///
/// Onboarding is done when:
/// 1. Both permissions (Input Monitoring + Accessibility) are granted, OR
/// 2. Input Monitoring (strictly required) is granted AND the user previously
///    completed/skipped onboarding (meaning they intentionally skipped Accessibility).
///
/// If Input Monitoring is missing, onboarding is NEVER done, even if previously
/// completed, because the app cannot function without it.
pub fn is_onboarding_done(
    input_granted: bool,
    ax_granted: bool,
    onboarding_completed: bool,
) -> bool {
    (input_granted && ax_granted) || (input_granted && onboarding_completed)
}

/// Check if a freshly spawned process has Input Monitoring access.
///
/// On macOS, granting Input Monitoring in System Settings sets the TCC
/// permission for the bundle, but macOS does NOT grant event listening to
/// an already-running process until that process terminates and relaunches.
/// Spawning a quick helper subprocess queries whether a new process instance
/// of this app will have Input Monitoring granted.
pub fn input_monitoring_granted_by_system() -> bool {
    if input_monitoring_granted() {
        return true;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    std::process::Command::new(exe)
        .arg("--check-input-monitoring")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Relaunch the application and terminate the current process.
pub fn relaunch_app() {
    if let Ok(exe) = std::env::current_exe() {
        let in_bundle = exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("Info.plist").exists())
            .unwrap_or(false);

        if in_bundle
            && let Some(bundle) = exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
        {
            let _ = std::process::Command::new("/usr/bin/open")
                .arg("-n")
                .arg(bundle)
                .spawn();
            std::process::exit(0);
        }

        let _ = std::process::Command::new(exe)
            .args(std::env::args().skip(1))
            .spawn();
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_onboarding_done() {
        // Fresh launch: neither granted
        assert!(!is_onboarding_done(false, false, false));
        assert!(!is_onboarding_done(false, false, true));

        // User granted input monitoring, quit & reopened, but hasn't completed onboarding yet
        assert!(!is_onboarding_done(true, false, false));

        // Both granted: always done
        assert!(is_onboarding_done(true, true, false));
        assert!(is_onboarding_done(true, true, true));

        // User completed onboarding previously and skipped accessibility
        assert!(is_onboarding_done(true, false, true));

        // Input monitoring revoked later: never done (required!)
        assert!(!is_onboarding_done(false, true, true));
    }

    #[test]
    fn settings_urls_point_at_the_right_panes() {
        let listen = Permission::InputMonitoring.settings_url();
        assert!(listen.contains("Privacy_ListenEvent"), "{listen}");
        assert!(listen.starts_with("x-apple.systempreferences:"), "{listen}");

        let ax = Permission::Accessibility.settings_url();
        assert!(ax.contains("Privacy_Accessibility"), "{ax}");
        assert!(ax.starts_with("x-apple.systempreferences:"), "{ax}");
    }
}

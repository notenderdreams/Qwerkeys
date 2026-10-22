use egui::Color32;

/// Linear-interpolate two colors; `t = 0` yields `a`, `t = 1` yields `b`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let lerp = |x: f32, y: f32| (x + (y - x) * t) as u8;
    Color32::from_rgba_unmultiplied(
        lerp(a.r() as f32, b.r() as f32),
        lerp(a.g() as f32, b.g() as f32),
        lerp(a.b() as f32, b.b() as f32),
        lerp(a.a() as f32, b.a() as f32),
    )
}

/// Embeds bytes of a file from the crate's `assets/` directory at compile time.
#[macro_export]
macro_rules! asset_bytes {
    ($file:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $file))
    };
}

/// Prints the clean startup banner before the GUI window opens.
pub fn print_banner(target: &str, is_suppressed: bool, port: &str) {
    println!("qwerkeys");
    println!("  Target:  {target}");
    println!(
        "  Mode:    {}",
        if is_suppressed {
            "Dedicated (keystrokes suppressed via CGEventTap)"
        } else {
            "Shared (internal / omni keyboard)"
        }
    );
    println!("  Port:    {port}");
    println!("Starting GUI window...\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_helper() {
        let bytes = asset_bytes!("app-icon.png");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_mix_colors() {
        let black = Color32::from_rgb(0, 0, 0);
        let white = Color32::from_rgb(255, 255, 255);
        assert_eq!(mix(black, white, 0.0), black);
        assert_eq!(mix(black, white, 1.0), white);
    }
}

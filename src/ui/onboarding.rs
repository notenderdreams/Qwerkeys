//! Onboarding page: welcome + macOS permission granting.
//!
//! Shown as a full-window page whenever Input Monitoring is not granted.
//! Presents each required/recommended permission as a themed card with a
//! live status pill; clicking a card triggers the TCC consent alert, and
//! the System Settings pane can be opened directly for manual grants.
//! The page polls permission state every 0.6s so the pills flip to
//! "GRANTED" the moment the user approves and returns.

use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Layout, Margin, Pos2, Rect,
    RichText, Sense, Stroke, Ui, vec2,
};
use egui_phosphor::regular::{ARROWS_CLOCKWISE, CHECK_CIRCLE, CURSOR, KEYBOARD, WARNING_CIRCLE};

use super::QwerkeysApp;
use crate::permissions::Permission;
use crate::utils::mix;

/// Amber used for the "needs access" pill; readable on all three themes.
const WARNING: Color32 = Color32::from_rgb(0xE3, 0x9D, 0x3A);

const CARD_W: f32 = 460.0;
const CARD_H: f32 = 92.0;
/// Padding around the two permission rows inside the merged container block.
const BLOCK_PAD: f32 = 6.0;

/// Total height of the centered stat panel stack (logo through skip); drives
/// the vertical centering offset. Keep in sync with the layout sums below.
const CONTENT_H: f32 = 473.0;

/// Static content of one permission card on the onboarding page.
struct PermissionCard {
    icon: &'static str,
    title: &'static str,
    requirement: &'static str,
    /// Description pre-folded into two short lines so it never runs under
    /// the status pill / link column on the right edge of the card.
    desc: [&'static str; 2],
    permission: Permission,
}

const PERMISSION_CARDS: [PermissionCard; 2] = [
    PermissionCard {
        icon: KEYBOARD,
        title: "Input Monitoring",
        requirement: "REQUIRED",
        desc: [
            "Reads keystrokes from your keyboard",
            "and converts them into MIDI notes.",
        ],
        permission: Permission::InputMonitoring,
    },
    PermissionCard {
        icon: CURSOR,
        title: "Accessibility",
        requirement: "RECOMMENDED",
        desc: [
            "Lets Qwerkeys swallow mapped keys so",
            "typing stays quiet while you play.",
        ],
        permission: Permission::Accessibility,
    },
];

impl QwerkeysApp {
    pub(super) fn render_onboarding(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(300));

        // If Input Monitoring is not yet held by this process, but was requested,
        // periodically check if a fresh subprocess has it (meaning the user toggled it
        // in System Settings, but macOS requires an app relaunch to apply it).
        let input_granted = Permission::InputMonitoring.is_granted();
        if !input_granted
            && self.onboarding_requested[0]
            && !self
                .input_monitoring_needs_restart
                .load(std::sync::atomic::Ordering::SeqCst)
            && self
                .last_restart_check
                .is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(800))
            && !self
                .restart_check_in_flight
                .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            self.last_restart_check = Some(std::time::Instant::now());
            let in_flight = std::sync::Arc::clone(&self.restart_check_in_flight);
            let needs_restart = std::sync::Arc::clone(&self.input_monitoring_needs_restart);
            let ctx = ui.ctx().clone();
            std::thread::spawn(move || {
                if crate::permissions::input_monitoring_granted_by_system() {
                    needs_restart.store(true, std::sync::atomic::Ordering::SeqCst);
                    ctx.request_repaint();
                }
                in_flight.store(false, std::sync::atomic::Ordering::SeqCst);
            });
        }

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme.BG_APP))
            .show(ui, |ui| {
                let view_h = ui.max_rect().height();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                            // Exact vertical centering of the fixed-height stack.
                            ui.add_space(((view_h - CONTENT_H) * 0.5).max(12.0));

                            // 1. Logo w/ version, right above the SETUP kicker
                            const LOGO_H: f32 = 76.0;
                            ui.add(
                                egui::Image::new(egui::include_image!("../../assets/logo.svg"))
                                    .fit_to_exact_size(vec2(316.0, LOGO_H))
                                    .tint(theme.LOGO),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                    .font(FontId::monospace(9.0))
                                    .strong()
                                    .color(theme.TEXT_LIGHT_MUTED),
                            );

                            ui.add_space(26.0);
                            ui.label(
                                RichText::new("SETUP")
                                    .font(FontId::monospace(9.5))
                                    .strong()
                                    .color(theme.TEXT_HEADING),
                            );

                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Welcome to Qwerkeys")
                                    .font(FontId::proportional(23.0))
                                    .strong()
                                    .color(theme.TEXT_PRIMARY),
                            );

                            // 2. Permission rows inside one rounded block
                            ui.add_space(16.0);
                            let block_w = CARD_W + 2.0 * BLOCK_PAD;
                            let block_h = 2.0 * CARD_H + 1.0 + 2.0 * BLOCK_PAD;
                            ui.allocate_ui_with_layout(
                                vec2(block_w, block_h),
                                Layout::top_down(Align::Center),
                                |ui| {
                                    Frame::NONE
                                        .fill(theme.BG_PANEL)
                                        .stroke(Stroke::new(1.2, theme.BORDER_PANEL))
                                        .corner_radius(CornerRadius::same(14))
                                        .inner_margin(Margin::same(BLOCK_PAD as i8))
                                        .show(ui, |ui| {
                                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                                            for (idx, card) in PERMISSION_CARDS.iter().enumerate() {
                                                if idx > 0 {
                                                    // Soft divider between the rows
                                                    let divider_color = mix(
                                                        theme.BG_PANEL,
                                                        theme.BORDER_FIELD,
                                                        0.35,
                                                    );
                                                    let (sep_rect, _) = ui.allocate_exact_size(
                                                        vec2(ui.available_width(), 1.0),
                                                        Sense::hover(),
                                                    );
                                                    ui.painter().hline(
                                                        sep_rect.x_range(),
                                                        sep_rect.center().y,
                                                        Stroke::new(1.0, divider_color),
                                                    );
                                                }
                                                self.render_permission_card(ui, card, idx);
                                            }
                                        });
                                },
                            );

                            // 3. Hint + continue button + skip
                            ui.add_space(16.0);
                            let needs_restart = self
                                .input_monitoring_needs_restart
                                .load(std::sync::atomic::Ordering::SeqCst);
                            let hint_text = if needs_restart {
                                "Input Monitoring granted in System Settings! Relaunch Qwerkeys to activate."
                            } else {
                                "Permissions can be changed anytime in System Settings \u{2192} Privacy & Security."
                            };
                            ui.label(
                                RichText::new(hint_text)
                                    .font(FontId::proportional(9.0))
                                    .color(if needs_restart {
                                        theme.SAGE_ACCENT
                                    } else {
                                        theme.TEXT_MUTED
                                    }),
                            );

                            ui.add_space(10.0);
                            let ready = Permission::InputMonitoring.is_granted()
                                && Permission::Accessibility.is_granted();
                            let (label, fill, txt) = if ready {
                                ("Get Started", theme.SAGE_ACCENT, theme.KEY_ACTIVE_TXT)
                            } else if needs_restart {
                                ("Relaunch to Apply", theme.SAGE_ACCENT, theme.KEY_ACTIVE_TXT)
                            } else {
                                ("Continue", theme.BG_BUTTON, theme.TEXT_PRIMARY)
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(label)
                                            .font(FontId::proportional(14.0))
                                            .strong()
                                            .color(txt),
                                    )
                                    .fill(fill)
                                    .stroke(Stroke::new(1.0, theme.BORDER_DARK))
                                    .corner_radius(CornerRadius::same(6))
                                    .min_size(vec2(230.0, 38.0)),
                                )
                                .clicked()
                            {
                                if needs_restart && !ready {
                                    crate::permissions::relaunch_app();
                                } else {
                                    self.finish_onboarding();
                                }
                            }

                            ui.add_space(8.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Skip for now")
                                            .font(FontId::proportional(10.5))
                                            .color(theme.TEXT_MUTED),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .min_size(vec2(230.0, 20.0)),
                                )
                                .clicked()
                            {
                                self.finish_onboarding();
                            }
                        });
                    });
            });
    }

    /// One permission row: icon tile, title + requirement chip, description,
    /// live status pill, and (after a prompt was declined) a deep link into
    /// the matching System Settings pane.
    fn render_permission_card(&mut self, ui: &mut Ui, card: &PermissionCard, idx: usize) {
        let PermissionCard {
            icon,
            title,
            requirement,
            desc,
            permission: perm,
        } = *card;
        let theme = self.theme_colors;
        let granted = perm.is_granted();
        let requested = self.onboarding_requested[idx];
        let needs_restart = idx == 0
            && !granted
            && self
                .input_monitoring_needs_restart
                .load(std::sync::atomic::Ordering::SeqCst);

        let (rect, response) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());
        let response = response.on_hover_cursor(if granted {
            CursorIcon::Default
        } else {
            CursorIcon::PointingHand
        });
        let hovered = response.hovered() && !granted;

        let painter = ui.painter_at(rect);

        // 1. Row container (inside the merged permission block)
        painter.rect(
            rect,
            CornerRadius::same(9),
            if hovered {
                theme.BG_FIELD
            } else {
                theme.BG_BUTTON
            },
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );

        // 2. Icon tile
        let icon_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + 16.0, rect.center().y - 20.0),
            vec2(40.0, 40.0),
        );
        painter.rect(
            icon_rect,
            CornerRadius::same(7),
            theme.SAGE_BADGE_BG,
            Stroke::new(0.8, theme.SAGE_BORDER),
            egui::StrokeKind::Inside,
        );
        painter.text(
            icon_rect.center(),
            Align2::CENTER_CENTER,
            icon,
            FontId::proportional(19.0),
            theme.SAGE_ACCENT,
        );

        // 3. Title + requirement chip
        let title_font = FontId::proportional(13.0);
        let title_pos = Pos2::new(rect.min.x + 72.0, rect.min.y + 31.0);
        let title_w = painter
            .layout_no_wrap(title.to_owned(), title_font.clone(), theme.TEXT_PRIMARY)
            .size()
            .x;
        painter.text(
            title_pos,
            Align2::LEFT_CENTER,
            title,
            title_font,
            theme.TEXT_PRIMARY,
        );

        let chip_font = FontId::monospace(8.0);
        let chip_w = painter
            .layout_no_wrap(
                requirement.to_owned(),
                chip_font.clone(),
                theme.SAGE_BADGE_TXT,
            )
            .size()
            .x
            + 14.0;
        let chip_rect = Rect::from_min_size(
            Pos2::new(title_pos.x + title_w + 8.0, title_pos.y - 9.0),
            vec2(chip_w, 18.0),
        );
        painter.rect(
            chip_rect,
            CornerRadius::same(9),
            theme.SAGE_BADGE_BG,
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        painter.text(
            chip_rect.center(),
            Align2::CENTER_CENTER,
            requirement,
            chip_font,
            theme.SAGE_BADGE_TXT,
        );

        // 4. Description (two folded lines, kept clear of the pill column)
        let desc_font = FontId::proportional(10.0);
        for (i, line) in desc.iter().enumerate() {
            painter.text(
                Pos2::new(rect.min.x + 72.0, rect.min.y + 48.0 + 14.0 * i as f32),
                Align2::LEFT_CENTER,
                *line,
                desc_font.clone(),
                theme.TEXT_MUTED,
            );
        }

        // 5. Status pill (live). Icon renders in the proportional family
        // (Phosphor fallback lives there) and the label in monospace; both
        // are measured so the pair stays optically centered.
        // Status pill: vertically centered on every row (consistent across
        // rows); the deep link below it belongs to the same right column.
        let pill_rect = Rect::from_min_size(
            Pos2::new(rect.max.x - 16.0 - 148.0, rect.center().y - 14.0),
            vec2(148.0, 28.0),
        );
        let (pill_bg, pill_txt, glyph, label) = if granted {
            (
                theme.SAGE_BADGE_BG,
                theme.SAGE_BADGE_TXT,
                CHECK_CIRCLE,
                "GRANTED",
            )
        } else if needs_restart {
            (
                theme.SAGE_BADGE_BG,
                theme.SAGE_ACCENT,
                ARROWS_CLOCKWISE,
                "RESTART TO APPLY",
            )
        } else {
            (
                mix(theme.BG_BUTTON, WARNING, 0.18),
                WARNING,
                WARNING_CIRCLE,
                "NEEDS ACCESS",
            )
        };
        painter.rect(
            pill_rect,
            CornerRadius::same(14),
            pill_bg,
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );

        let label_font = FontId::monospace(10.0);
        let glyph_font = FontId::proportional(14.0);
        let glyph_w = painter
            .layout_no_wrap(glyph.to_owned(), glyph_font.clone(), pill_txt)
            .size()
            .x;
        let label_w = painter
            .layout_no_wrap(label.to_owned(), label_font.clone(), pill_txt)
            .size()
            .x;
        let gap = 5.0;
        let start_x = pill_rect.center().x - (glyph_w + gap + label_w) * 0.5;
        let mid_y = pill_rect.center().y;
        painter.text(
            Pos2::new(start_x + glyph_w, mid_y),
            Align2::RIGHT_CENTER,
            glyph,
            glyph_font,
            pill_txt,
        );
        painter.text(
            Pos2::new(start_x + glyph_w + gap, mid_y),
            Align2::LEFT_CENTER,
            label,
            label_font,
            pill_txt,
        );

        // 6. Manual-grant deep link, shown after a prompt didn't grant,
        // or relaunch action if granted by system but needs app restart.
        let link_font = FontId::monospace(8.5);
        let link_rect = Rect::from_min_size(
            Pos2::new(rect.max.x - 166.0, pill_rect.max.y + 10.0),
            vec2(150.0, 16.0),
        );
        let show_link = (requested && !granted) || needs_restart;
        if show_link {
            let hovering = ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| link_rect.contains(p));
            let link_color = if hovering {
                mix(theme.SAGE_ACCENT, theme.TEXT_PRIMARY, 0.35)
            } else {
                theme.SAGE_ACCENT
            };
            let link_text = if needs_restart {
                "RELAUNCH APP NOW"
            } else {
                "OPEN SYSTEM SETTINGS"
            };
            painter.text(
                Pos2::new(rect.max.x - 16.0, link_rect.center().y),
                Align2::RIGHT_CENTER,
                link_text,
                link_font,
                link_color,
            );
        }

        // 7. Interaction: card click = consent prompt + open the matching
        // System Settings pane, or relaunch if restart is needed.
        if response.clicked() {
            if granted {
                // nothing to do
            } else if needs_restart {
                crate::permissions::relaunch_app();
            } else {
                perm.request();
                perm.open_system_settings();
                self.onboarding_requested[idx] = true;
            }
        }
    }

    /// Close onboarding and kick off the deferred HID listener (if the
    /// listener startup was held back pending permissions).
    fn finish_onboarding(&mut self) {
        self.onboarding_done = true;
        self.state
            .onboarding_completed
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.persist();
        if let Some(start) = self.start_listener.take() {
            start();
        }
    }
}

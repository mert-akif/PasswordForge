use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{self, RichText, Ui};

use crate::{
    generator::{PasswordOptions, generate_password},
    theme::{self, Palette},
    typing,
};

const COUNTDOWN_SECONDS: u64 = 5;

#[derive(Clone, Copy)]
enum StatusKind {
    Info,
    Success,
    Error,
}

struct StatusMessage {
    text: String,
    kind: StatusKind,
}

struct TypingJob {
    began_at: Instant,
    cancel: Arc<AtomicBool>,
    result_rx: Receiver<Result<(), String>>,
}

pub struct PasswordApp {
    // Password generator settings.
    options: PasswordOptions,
    generator_open: bool,
    generator_error: Option<String>,

    // Single editable password.
    password: String,
    show_password: bool,

    // Application state.
    dark_mode: bool,
    status: Option<StatusMessage>,
    typing_job: Option<TypingJob>,
}

impl Default for PasswordApp {
    fn default() -> Self {
        Self {
            options: PasswordOptions::default(),
            generator_open: false,
            generator_error: None,

            password: String::new(),
            show_password: false,

            dark_mode: true,
            status: None,
            typing_job: None,
        }
    }
}

impl PasswordApp {
    /// Initialize the application theme.
    pub fn new(ctx: &egui::Context) -> Self {
        let app = Self::default();

        theme::apply_theme(ctx, app.dark_mode);

        app
    }

    /// Set a message in the status panel.
    fn set_status(
        &mut self,
        kind: StatusKind,
        message: impl Into<String>,
    ) {
        self.status = Some(StatusMessage {
            text: message.into(),
            kind,
        });
    }

    /// Generate a password and replace the editor content.
    fn generate(&mut self) -> Result<(), String> {
        match generate_password(&self.options) {
            Ok(password) => {
                self.password = password;

                // Always hide newly generated passwords.
                self.show_password = false;

                self.set_status(
                    StatusKind::Success,
                    "Password generated successfully.",
                );

                Ok(())
            }

            Err(error) => {
                // Preserve the previously entered password.
                self.set_status(
                    StatusKind::Error,
                    error.clone(),
                );

                Err(error)
            }
        }
    }

    /// Start Auto-Type using the current editor content.
    fn start_auto_type(&mut self) {
        if self.typing_job.is_some() {
            return;
        }

        if self.password.is_empty() {
            self.set_status(
                StatusKind::Error,
                "Enter or generate a password first.",
            );
            return;
        }

        if self.password.chars().any(char::is_control) {
            self.set_status(
                StatusKind::Error,
                "Control characters cannot be auto-typed.",
            );
            return;
        }

        if let Err(error) = typing::check_platform() {
            self.set_status(StatusKind::Error, error);
            return;
        }

        // Snapshot the password before starting the worker.
        let password = self.password.clone();

        let (result_tx, result_rx) = mpsc::channel();

        let cancel = Arc::new(AtomicBool::new(false));
        let thread_cancel = Arc::clone(&cancel);

        self.typing_job = Some(TypingJob {
            began_at: Instant::now(),
            cancel,
            result_rx,
        });

        self.set_status(
            StatusKind::Info,
            "Switch to the target window and focus the input field.",
        );

        thread::spawn(move || {
            // Five-second countdown.
            // Check cancellation every 100 milliseconds.
            for _ in 0..(COUNTDOWN_SECONDS * 10) {
                if thread_cancel.load(Ordering::Relaxed) {
                    let _ = result_tx.send(
                        Err("Auto-Type cancelled.".to_owned())
                    );
                    return;
                }

                thread::sleep(
                    Duration::from_millis(100)
                );
            }

            // Check cancellation before typing.
            if thread_cancel.load(Ordering::Relaxed) {
                let _ = result_tx.send(
                    Err("Auto-Type cancelled.".to_owned())
                );
                return;
            }

            let result = typing::type_password(
                &password,
                &thread_cancel,
            );

            let _ = result_tx.send(result);
        });
    }

    /// Read the result of the background typing operation.
    fn poll_auto_type(&mut self, ctx: &egui::Context) {
        let result = match self.typing_job.as_ref() {
            None => return,

            Some(job) => match job.result_rx.try_recv() {
                Ok(result) => Some(result),

                Err(TryRecvError::Empty) => None,

                Err(TryRecvError::Disconnected) => {
                    Some(Err(
                        "Auto-Type stopped unexpectedly."
                            .to_owned()
                    ))
                }
            },
        };

        if let Some(result) = result {
            match result {
                Ok(()) => {
                    self.set_status(
                        StatusKind::Success,
                        "Keyboard input sent. Verify the target field.",
                    );
                }

                Err(error) => {
                    self.set_status(
                        StatusKind::Error,
                        error,
                    );
                }
            }

            self.typing_job = None;
        } else {
            ctx.request_repaint_after(
                Duration::from_millis(100)
            );
        }
    }

    /// Request cancellation of the active typing job.
    fn cancel_auto_type(&mut self) {
        if let Some(job) = &self.typing_job {
            job.cancel.store(
                true,
                Ordering::Relaxed,
            );

            self.set_status(
                StatusKind::Info,
                "Cancellation requested.",
            );
        }
    }

    /// Draw the application title and theme switch.
    fn draw_header(&mut self, ui: &mut Ui, p: &Palette) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("PASSWORD FORGE")
                        .size(28.0)
                        .strong()
                        .color(p.text),
                );

                ui.label(
                    RichText::new(
                        "Secure password generation & automatic typing"
                    )
                        .size(13.0)
                        .color(p.muted),
                );
            });

            ui.with_layout(
                egui::Layout::right_to_left(
                    egui::Align::Center
                ),
                |ui| {
                    let label = if self.dark_mode {
                        "Light Mode"
                    } else {
                        "Dark Mode"
                    };

                    if ui.add(
                        theme::secondary_button(label)
                    ).clicked() {
                        self.dark_mode = !self.dark_mode;

                        theme::apply_theme(
                            ui.ctx(),
                            self.dark_mode,
                        );

                        ui.ctx().request_repaint();
                    }
                },
            );
        });

        ui.add_space(12.0);
    }

    /// Draw the single editable password field.
    fn draw_password_editor(
        &mut self,
        ui: &mut Ui,
        p: &Palette,
        is_busy: bool,
    ) {
        let content_width =
            (ui.available_width() - 42.0).max(220.0);

        theme::card(p).show(ui, |ui| {
            ui.set_min_width(content_width);

            ui.label(
                RichText::new("Password Editor")
                    .size(20.0)
                    .strong()
                    .color(p.text),
            );

            ui.label(
                RichText::new(
                    "Enter your own password or generate a new one."
                )
                    .size(12.0)
                    .color(p.muted),
            );

            ui.add_space(12.0);

            ui.label(
                RichText::new("PASSWORD")
                    .size(11.0)
                    .strong()
                    .color(p.muted),
            );

            // The only password TextEdit in the application.
            ui.add_enabled(
                !is_busy,
                egui::TextEdit::singleline(
                    &mut self.password
                )
                    .password(!self.show_password)
                    .hint_text(
                        "Enter or generate a password..."
                    )
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(5.0);

            // Password information and secondary actions.
            ui.horizontal_wrapped(|ui| {
                let char_count =
                    self.password.chars().count();

                ui.label(
                    RichText::new(
                        format!("{char_count} characters")
                    )
                        .size(12.0)
                        .color(p.muted),
                );

                ui.separator();

                let visibility_label =
                    if self.show_password {
                        "Hide"
                    } else {
                        "Show"
                    };

                if ui.add(
                    theme::secondary_button(
                        visibility_label
                    ),
                ).clicked() {
                    self.show_password =
                        !self.show_password;
                }

                if ui.add_enabled(
                    !self.password.is_empty(),
                    theme::secondary_button("Copy"),
                ).clicked() {
                    ui.ctx().copy_text(
                        self.password.clone()
                    );

                    self.set_status(
                        StatusKind::Success,
                        "Password copy requested.",
                    );
                }
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Only two primary application actions.
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(
                    !is_busy,
                    theme::primary_button(
                        "Generate Password",
                        p,
                    ),
                ).clicked() {
                    self.generator_error = None;
                    self.generator_open = true;
                }

                if ui.add_enabled(
                    !is_busy &&
                        !self.password.is_empty(),
                    theme::primary_button(
                        "Auto-Type",
                        p,
                    ),
                ).clicked() {
                    self.start_auto_type();
                }
            });

            ui.add_space(5.0);
        });
    }

    /// Display generator settings in a separate window.
    fn draw_generator_window(
        &mut self,
        ctx: &egui::Context,
        p: &Palette,
        is_busy: bool,
    ) {
        if !self.generator_open {
            return;
        }

        let mut window_open = self.generator_open;
        let mut generate_requested = false;
        let mut close_requested = false;

        egui::Window::new("Password Generator")
            .open(&mut window_open)
            .collapsible(false)
            .resizable(false)
            .default_width(440.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(
                        "Configure your new password"
                    )
                        .size(18.0)
                        .strong()
                        .color(p.text),
                );

                ui.label(
                    RichText::new(
                        "Generating a password will replace the current editor content."
                    )
                        .size(12.0)
                        .color(p.muted),
                );

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.label("Password Length");

                    ui.label(
                        RichText::new(
                            format!("{}", self.options.length)
                        )
                            .strong()
                            .color(p.accent),
                    );
                });

                ui.add_enabled(
                    !is_busy,
                    egui::Slider::new(
                        &mut self.options.length,
                        8..=64,
                    )
                        .show_value(false),
                );

                ui.add_space(8.0);

                ui.add_enabled_ui(!is_busy, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].checkbox(
                            &mut self.options.lowercase,
                            "Lowercase (a-z)",
                        );

                        columns[1].checkbox(
                            &mut self.options.uppercase,
                            "Uppercase (A-Z)",
                        );

                        columns[0].checkbox(
                            &mut self.options.digits,
                            "Numbers (0-9)",
                        );

                        columns[1].checkbox(
                            &mut self.options.symbols,
                            "Special Characters",
                        );
                    });

                    ui.add_space(10.0);

                    ui.label(
                        RichText::new(
                            "Additional Characters"
                        )
                            .strong(),
                    );

                    ui.add_sized(
                        [ui.available_width(), 36.0],
                        egui::TextEdit::singleline(
                            &mut self.options.custom_chars
                        )
                            .hint_text(
                                "Optional custom characters"
                            ),
                    );
                });

                if let Some(error) =
                    &self.generator_error
                {
                    ui.add_space(8.0);

                    ui.label(
                        RichText::new(error)
                            .color(p.error),
                    );
                }

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if ui.add_enabled(
                        !is_busy,
                        theme::primary_button(
                            "Generate & Replace",
                            p,
                        ),
                    ).clicked() {
                        generate_requested = true;
                    }

                    if ui.add(
                        theme::secondary_button(
                            "Cancel"
                        ),
                    ).clicked() {
                        close_requested = true;
                    }
                });
            });

        self.generator_open =
            window_open && !close_requested;

        if generate_requested {
            match self.generate() {
                Ok(()) => {
                    self.generator_open = false;
                    self.generator_error = None;
                }

                Err(error) => {
                    self.generator_error = Some(error);
                }
            }
        }
    }

    /// Draw the current status and typing progress.
    fn draw_status(&mut self, ui: &mut Ui, p: &Palette) {
        let (message, color) = match &self.status {
            Some(status) => {
                let color = match status.kind {
                    StatusKind::Info => p.accent,
                    StatusKind::Success => p.success,
                    StatusKind::Error => p.error,
                };

                (status.text.as_str(), color)
            }

            None => (
                "Ready to generate or enter a password.",
                p.success,
            ),
        };

        egui::Frame::new()
            .fill(p.surface)
            .stroke(
                egui::Stroke::new(1.0, p.border)
            )
            .corner_radius(10)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("●")
                            .color(color)
                            .size(14.0),
                    );

                    ui.label(
                        RichText::new(message)
                            .color(p.text),
                    );
                });
            });

        if let Some(job) = &self.typing_job {
            let remaining =
                COUNTDOWN_SECONDS.saturating_sub(
                    job.began_at.elapsed().as_secs()
                );

            ui.add_space(10.0);

            if remaining > 0 {
                ui.label(
                    RichText::new(
                        format!(
                            "Auto-Type starts in {remaining} second(s)..."
                        )
                    )
                        .color(p.accent),
                );
            } else {
                ui.label(
                    RichText::new(
                        "Sending keyboard input..."
                    )
                        .color(p.accent),
                );
            }

            if ui.button(
                "Cancel Auto-Type"
            ).clicked() {
                self.cancel_auto_type();
            }
        }
    }
}

impl Drop for PasswordApp {
    fn drop(&mut self) {
        if let Some(job) = &self.typing_job {
            job.cancel.store(
                true,
                Ordering::Relaxed,
            );
        }
    }
}

impl eframe::App for PasswordApp {
    fn ui(
        &mut self,
        ui: &mut Ui,
        _frame: &mut eframe::Frame,
    ) {
        self.poll_auto_type(ui.ctx());

        let is_busy = self.typing_job.is_some();
        let palette = Palette::new(self.dark_mode);

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(12.0);

                self.draw_header(
                    ui,
                    &palette,
                );

                self.draw_password_editor(
                    ui,
                    &palette,
                    is_busy,
                );

                ui.add_space(18.0);

                self.draw_status(
                    ui,
                    &palette,
                );

                ui.add_space(16.0);

                ui.label(
                    RichText::new(
                        "Auto-Type sends input to the currently focused field. Verify the destination before starting."
                    )
                        .size(11.0)
                        .color(palette.muted),
                );

                ui.add_space(6.0);

                ui.label(
                    RichText::new(
                        "Version 0.3 • Built with Rust & egui"
                    )
                        .size(11.0)
                        .color(palette.muted),
                );

                ui.add_space(12.0);
            });
        });

        // Draw the generator popup above the main UI.
        self.draw_generator_window(
            ui.ctx(),
            &palette,
            is_busy,
        );
    }
}
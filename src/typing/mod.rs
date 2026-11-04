use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use enigo::{Enigo, Keyboard, Settings};

#[cfg(not(target_os = "windows"))]
use enigo::{Direction, Key};

/// Platform-independent interface for automatic typing.
pub trait AutoTyper {
    fn type_char(
        &mut self,
        character: char,
    ) -> Result<(), String>;
}

/// Enigo-based keyboard input implementation.
struct EnigoTyper {
    enigo: Enigo,
}

impl AutoTyper for EnigoTyper {
    fn type_char(
        &mut self,
        character: char,
    ) -> Result<(), String> {

        // Windows: send characters as Unicode text.
        // This avoids keyboard layout-dependent key mapping.
        #[cfg(target_os = "windows")]
        {
            self.enigo
                .text(&character.to_string())
                .map_err(|error| {
                    format!(
                        "Failed to type character: {error}"
                    )
                })?;
        }

        // Linux and macOS: preserve the existing
        // individual key event implementation.
        #[cfg(not(target_os = "windows"))]
        {
            self.enigo
                .key(
                    Key::Unicode(character),
                    Direction::Click,
                )
                .map_err(|error| {
                    format!(
                        "Failed to send keyboard input: {error}"
                    )
                })?;
        }

        Ok(())
    }
}

/// Check whether Auto-Type is available.
pub fn check_platform() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("XDG_SESSION_TYPE")
            .is_ok_and(|session| {
                session.eq_ignore_ascii_case("wayland")
            })
        {
            return Err(
                "Wayland detected. Auto-Type is currently supported only on X11."
                    .to_owned()
            );
        }

        if std::env::var_os("DISPLAY").is_none() {
            return Err(
                "No X11 display was found. Auto-Type is unavailable."
                    .to_owned()
            );
        }
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "macos"
    )))]
    {
        return Err(
            "Auto-Type is not supported on this operating system."
                .to_owned()
        );
    }

    Ok(())
}

/// Initialize the platform-specific keyboard backend.
fn make_typer() -> Result<Box<dyn AutoTyper>, String> {
    check_platform()?;

    let enigo = Enigo::new(&Settings::default())
        .map_err(|error| {
            format!(
                "Failed to initialize keyboard backend: {error}"
            )
        })?;

    Ok(Box::new(EnigoTyper { enigo }))
}

/// Returns the typing delay for the current platform.
fn typing_delay() -> Duration {

    Duration::from_millis(40)

}

/// Type the password one character at a time.
fn type_all(
    typer: &mut dyn AutoTyper,
    password: &str,
    cancelled: &AtomicBool,
    delay: Duration,
) -> Result<(), String> {

    for character in password.chars() {

        // Stop if cancellation was requested.
        if cancelled.load(Ordering::Relaxed) {
            return Err(
                "Auto-Type was cancelled.".to_owned()
            );
        }

        // Send the next character.
        typer.type_char(character)?;

        // Give the target application time
        // to process the previous input.
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    Ok(())
}

/// Automatically type a password into the focused field.
///
/// This function should be executed on a worker thread.
pub fn type_password(
    password: &str,
    cancelled: &AtomicBool,
) -> Result<(), String> {

    if password.is_empty() {
        return Err(
            "There is no password to type.".to_owned()
        );
    }

    let mut typer = make_typer()?;

    type_all(
        typer.as_mut(),
        password,
        cancelled,
        typing_delay(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeTyper {
        typed: String,
    }

    impl AutoTyper for FakeTyper {
        fn type_char(
            &mut self,
            character: char,
        ) -> Result<(), String> {
            self.typed.push(character);
            Ok(())
        }
    }

    #[test]
    fn sends_characters_in_order() {
        let mut fake = FakeTyper::default();
        let cancelled = AtomicBool::new(false);

        type_all(
            &mut fake,
            "A9ğ!@#",
            &cancelled,
            Duration::ZERO,
        )
            .unwrap();

        assert_eq!(fake.typed, "A9ğ!@#");
    }

    #[test]
    fn cancellation_prevents_input() {
        let mut fake = FakeTyper::default();
        let cancelled = AtomicBool::new(true);

        assert!(
            type_all(
                &mut fake,
                "secret",
                &cancelled,
                Duration::ZERO,
            )
                .is_err()
        );

        assert!(fake.typed.is_empty());
    }
}
# Password Forge

*A secure, customizable password generator and automatic typing desktop application built entirely in Rust.*

Password Forge is an open-source desktop application that allows users to generate cryptographically secure passwords, manually edit them, copy them to the clipboard, and automatically type them into a focused input field.

The application is built entirely with Rust, using egui and eframe for its native graphical interface.

No JavaScript, web framework, or external server is required.

---

## Features

### Password Generator

- Cryptographically secure password generation
- Configurable password length (8–64 characters)
- Uppercase letters (A–Z)
- Lowercase letters (a–z)
- Numbers (0–9)
- Special characters
- Custom Unicode characters
- At least one character from each enabled group
- Randomized character order

### Password Editor

- Single editable password field
- Enter your own password manually
- Generate a password and modify it afterward
- Passwords hidden by default
- Show/Hide password functionality
- Copy passwords to the system clipboard
- Display password character count

### Auto-Type

- Automatically types the current password into a focused input field
- Does not rely on standard clipboard paste commands
- Five-second countdown before typing
- Cancellation support
- Background execution to prevent GUI freezing
- Platform-specific keyboard input handling
- Supports both generated and manually entered passwords

Auto-Type can be useful when an application restricts conventional clipboard pasting.

*Note:* Auto-Type is experimental. Some applications may reject simulated keyboard input, and compatibility is not guaranteed.

### User Interface

- Native desktop application
- Dark and light themes
- Minimalist interface
- Separate password generation settings window
- Status messages and operation feedback
- Responsive layout with scrolling support

---

## Technology Stack

| Component | Technology |
|---|---|
| Programming Language | Rust |
| GUI Framework | egui |
| Desktop Framework | eframe |
| Secure Randomness | getrandom |
| Keyboard Simulation | enigo |
| Threading | Rust Standard Library |
| Communication | std::sync::mpsc |
| Synchronization | Arc / AtomicBool |

The application uses a modular architecture to separate password generation, keyboard simulation, graphical interface, and styling.

---

## Platform Compatibility

| Platform | Password Generation | Auto-Type |
|---|---|---|
| Windows 10/11 | Supported | Experimental |
| Linux (X11) | Supported | Experimental |
| Linux (Wayland) | Supported | Not implemented |
| macOS | Intended support | Not verified |

The application has been developed and tested primarily on Windows and Linux.

Auto-Type behavior depends on the operating system, active keyboard layout, permissions, and target application.

### Windows

The Auto-Type implementation uses Enigo's Unicode text input functionality.

Windows Smart App Control may block locally compiled or unsigned executables. This does not necessarily indicate that the application contains malicious software.

### Linux

Auto-Type currently targets X11 sessions.

On Linux, check your current session type:

bash
echo "$XDG_SESSION_TYPE"


If the output is wayland, Auto-Type is disabled by the application's current implementation.

### macOS

macOS support has not yet been fully tested.

Keyboard simulation may require Accessibility permissions.

---

## Installation

### Requirements

To build Password Forge from source, you need:

- Rust and Cargo
- A compatible Rust toolchain
- Platform-specific native build dependencies
- Microsoft C++ Build Tools and Windows SDK when using the Windows MSVC target

Install Rust from the official website:

https://rustup.rs/

### Clone the Repository

bash
git clone https://github.com/mert-akif/PasswordForge.git
cd password-forge


### Run the Application

bash
cargo run


### Build a Release Version

bash
cargo build --release


The compiled executable will be located in:

*Windows:*

text
target/release/<package-name>.exe


*Linux:*

text
target/release/<package-name>


The executable name depends on the package name specified in Cargo.toml.

### Run Tests

bash
cargo test


### Check Code Formatting

bash
cargo fmt --check


### Run Clippy

bash
cargo clippy --all-targets


---

## How to Use

### 1. Enter a Password

Launch Password Forge.

The main window contains a single password editor.

You can enter a password manually or generate a new one.

Passwords are hidden by default. Click *Show* to reveal the password, or *Hide* to conceal it again.

### 2. Generate a Password

1. Click *Generate Password*.
2. The password generator settings window will open.
3. Select a password length.
4. Enable your preferred character groups.
5. Optionally specify additional characters.
6. Click *Generate & Replace*.

The generated password automatically replaces the current content of the password editor.

You can modify the generated password afterward.

### 3. Copy a Password

Click *Copy* to send the current password to the system clipboard.

The copied password can then be pasted into another application.

### 4. Automatically Type a Password

1. Enter or generate a password.
2. Click *Auto-Type*.
3. Switch to your target application.
4. Select the intended input field.
5. Wait for the five-second countdown.
6. Password Forge attempts to type the password automatically.

The application sends keyboard input rather than using the normal clipboard paste command.

You can request cancellation during the countdown or typing process.

*Important:* Auto-Type does not verify which application or input field is currently focused. Always confirm the destination before starting.

Cancellation cannot undo characters that have already been sent.

---

## Project Structure

text
password-forge/
├── .github/
│   └── workflows/
│       └── windows-build.yml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── theme.rs
│   ├── generator/
│   │   └── mod.rs
│   └── typing/
│       └── mod.rs
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── LICENSE
└── README.md


### Modules

*main.rs*

Initializes the desktop application and its native window.

*app.rs*

Manages the GUI, password editor, generator settings, Auto-Type countdown, and background operations.

*theme.rs*

Defines the application's color palettes, button styles, spacing, and dark/light theme support.

*generator/mod.rs*

Implements password generation using operating-system-backed cryptographic randomness.

*typing/mod.rs*

Provides the AutoTyper interface and keyboard simulation implementation.

---

## Security Considerations

Password Forge is designed to generate strong random passwords while keeping its functionality local to the user's computer.

### Password Generation

Random values are obtained through the operating system's cryptographic randomness facilities using the getrandom crate.

The generator uses rejection sampling to avoid modulo bias when selecting characters.

### Local Processing

- Password generation runs locally.
- No account registration is required.
- No cloud service is required.
- Passwords are not intentionally transmitted over a network.
- Passwords are not intentionally stored in files.

### Limitations

- Passwords may remain temporarily in application memory.
- Secure memory erasure is not guaranteed.
- Clipboard managers may retain copied passwords.
- Auto-Type may send sensitive text to the wrong focused application.
- Simulated keyboard input may not work in every application.
- Auto-Type cancellation cannot reverse previously transmitted input.

*Password Forge has not undergone an independent security audit.*

Users should review the source code and understand these limitations before using the application for sensitive credentials.

---

## Development Roadmap

### Completed

- [x] Cryptographically secure password generation
- [x] Customizable character sets
- [x] Custom Unicode characters
- [x] Single editable password field
- [x] Password visibility toggle
- [x] Clipboard support
- [x] Windows Auto-Type implementation
- [x] Linux X11 Auto-Type implementation
- [x] Auto-Type countdown
- [x] Auto-Type cancellation
- [x] Dark and light themes
- [x] Modular Rust architecture
- [x] Unit tests

### Planned

- [ ] Linux Wayland Auto-Type support
- [ ] Global keyboard shortcuts
- [ ] Configurable Auto-Type delay
- [ ] Persistent application settings
- [ ] Improved keyboard layout compatibility
- [ ] Clipboard auto-clear option
- [ ] Application icon
- [ ] Windows and Linux release packages
- [ ] Automated cross-platform testing
- [ ] macOS compatibility testing

---

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

If you encounter a problem, please open a GitHub Issue and include:

- Operating system and version
- Application version
- Steps to reproduce the issue
- Expected behavior
- Actual behavior

Please do not include real passwords or other sensitive information in bug reports.

For significant changes, consider opening an issue before submitting a pull request.

---

## License

This project is licensed under the MIT License.

See the [LICENSE](LICENSE) file for the full license text.

---

*Built with Rust, egui, and eframe.*

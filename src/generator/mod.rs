const SYMBOLS: &str = "!@#$%^&*()-_=+[]{}?";

#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: usize,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub custom_chars: String,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            custom_chars: String::new(),
        }
    }
}

pub fn generate_password(
    options: &PasswordOptions,
) -> Result<String, String> {
    if !(8..=64).contains(&options.length) {
        return Err(
            "Password length must be between 8 and 64."
                .to_owned()
        );
    }

    // Reject control characters such as Tab, Enter,
    // and NUL to prevent unintended input behavior.
    if options
        .custom_chars
        .chars()
        .any(|ch| ch.is_control())
    {
        return Err(
            "Custom characters cannot contain control characters."
                .to_owned()
        );
    }

    // Store each enabled character group separately.
    let mut groups: Vec<Vec<char>> = Vec::new();

    if options.lowercase {
        groups.push(
            "abcdefghijklmnopqrstuvwxyz"
                .chars()
                .collect()
        );
    }

    if options.uppercase {
        groups.push(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .collect()
        );
    }

    if options.digits {
        groups.push(
            "0123456789".chars().collect()
        );
    }

    if options.symbols {
        groups.push(
            SYMBOLS.chars().collect()
        );
    }

    if !options.custom_chars.is_empty() {
        let mut custom_group = Vec::new();

        // Remove duplicate custom characters.
        for ch in options.custom_chars.chars() {
            if !custom_group.contains(&ch) {
                custom_group.push(ch);
            }
        }

        groups.push(custom_group);
    }

    if groups.is_empty() {
        return Err(
            "Select at least one character group."
                .to_owned()
        );
    }

    if groups.len() > options.length {
        return Err(
            "The number of selected character groups exceeds the password length."
                .to_owned()
        );
    }

    // Create a combined alphabet without duplicates.
    let mut alphabet = Vec::new();

    for group in &groups {
        for &ch in group {
            if !alphabet.contains(&ch) {
                alphabet.push(ch);
            }
        }
    }

    let mut output = Vec::with_capacity(options.length);

    // Guarantee at least one character from each group.
    for group in &groups {
        output.push(choose(group)?);
    }

    // Fill the remaining positions.
    while output.len() < options.length {
        output.push(choose(&alphabet)?);
    }

    // Fisher-Yates shuffle using secure random indices.
    for index in (1..output.len()).rev() {
        let swap_index = secure_index(index + 1)?;
        output.swap(index, swap_index);
    }

    Ok(output.into_iter().collect())
}

fn choose(
    characters: &[char],
) -> Result<char, String> {
    let index = secure_index(characters.len())?;
    Ok(characters[index])
}

fn secure_index(
    upper: usize,
) -> Result<usize, String> {
    let bound = u64::try_from(upper)
        .map_err(|_| {
            "Character pool is too large.".to_owned()
        })?;

    if bound == 0 {
        return Err(
            "Character pool cannot be empty.".to_owned()
        );
    }

    // Rejection sampling prevents modulo bias.
    let limit = u64::MAX - (u64::MAX % bound);

    loop {
        let mut bytes = [0_u8; 8];

        getrandom::fill(&mut bytes)
            .map_err(|error| {
                format!(
                    "Failed to obtain secure random data: {error}"
                )
            })?;

        let value = u64::from_ne_bytes(bytes);

        if value < limit {
            return Ok((value % bound) as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_password_has_correct_length_and_groups() {
        let options = PasswordOptions {
            length: 30,
            custom_chars: "ğβ".to_owned(),
            ..PasswordOptions::default()
        };

        let password =
            generate_password(&options).unwrap();

        assert_eq!(password.chars().count(), 30);

        assert!(
            password.chars().any(|c| c.is_ascii_lowercase())
        );

        assert!(
            password.chars().any(|c| c.is_ascii_uppercase())
        );

        assert!(
            password.chars().any(|c| c.is_ascii_digit())
        );

        assert!(
            password.chars().any(|c| SYMBOLS.contains(c))
        );

        assert!(
            password.contains('ğ') || password.contains('β')
        );
    }

    #[test]
    fn rejects_control_characters() {
        let options = PasswordOptions {
            custom_chars: "X\n".to_owned(),
            ..PasswordOptions::default()
        };

        assert!(
            generate_password(&options).is_err()
        );
    }

    #[test]
    fn rejects_empty_character_selection() {
        let options = PasswordOptions {
            lowercase: false,
            uppercase: false,
            digits: false,
            symbols: false,
            ..PasswordOptions::default()
        };

        assert!(
            generate_password(&options).is_err()
        );
    }

    #[test]
    fn only_custom_characters_when_other_groups_are_disabled() {
        let options = PasswordOptions {
            lowercase: false,
            uppercase: false,
            digits: false,
            symbols: false,
            custom_chars: "XY".to_owned(),
            ..PasswordOptions::default()
        };

        let password =
            generate_password(&options).unwrap();

        assert!(
            password.chars().all(|c| c == 'X' || c == 'Y')
        );
    }
}
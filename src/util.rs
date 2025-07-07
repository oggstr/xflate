use std::str::Chars;

use itertools::MultiPeek;

/// Consume characters from the `MultiPeek<Chars>`
/// iterator until a whitespace character is encountered.
/// Returns the consumed characters as a `String`.
pub fn consume_until_whitespace(chars: &mut MultiPeek<Chars<'_>>) -> String {
    let mut result: String = String::new();
    while let Some(c) = chars.peek().cloned() {
        if c.is_whitespace() {
            // Avoid side effects
            chars.reset_peek();
            break;
        }
        chars.next(); // Consume the character
        result.push(c);
    }

    result
}

pub fn consume_until_space(chars: &mut MultiPeek<Chars<'_>>) -> String {
    let mut result: String = String::new();
    while let Some(c) = chars.peek().cloned() {
        if c == ' ' {
            // Avoid side effects
            chars.reset_peek();
            break;
        }
        chars.next(); // Consume the character
        result.push(c);
    }

    result
}

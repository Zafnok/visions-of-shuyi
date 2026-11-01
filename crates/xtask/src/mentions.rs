//! Finds what GitHub would read as a mention of a user (ticket 0124): the
//! release notes are generated from PR titles, so a bare at-word in one lists
//! a stranger as a contributor.

/// Each bare mention in `text`, with its at sign, in order. A bare mention is
/// an at sign followed by an ASCII letter or digit, not right after an ASCII
/// letter, digit or `_` (an email address), and not inside a code span.
pub fn bare_mentions(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut in_code = false;
    let mut previous = ' ';
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        previous = if c == '`' {
            in_code = !in_code;
            c
        } else if c == '@'
            && !in_code
            && !(previous.is_ascii_alphanumeric() || previous == '_')
            && chars.peek().is_some_and(char::is_ascii_alphanumeric)
        {
            let mut mention = String::from(c);
            while let Some(next) = chars.next_if(|n| n.is_ascii_alphanumeric() || *n == '-') {
                mention.push(next);
            }
            // The run just read is what the next character comes right after.
            let last = mention.chars().last().unwrap_or(c);
            found.push(mention);
            last
        } else {
            c
        };
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: [&str; 0] = [];

    #[test]
    fn finds_a_mention_at_the_end_the_start_and_the_middle() {
        assert_eq!(bare_mentions("choose the music: @music"), ["@music"]);
        assert_eq!(bare_mentions("@music chooses"), ["@music"]);
        assert_eq!(bare_mentions("the @music directive"), ["@music"]);
    }

    #[test]
    fn finds_two_mentions_in_order() {
        assert_eq!(bare_mentions("@zeta then @alpha"), ["@zeta", "@alpha"]);
    }

    #[test]
    fn a_mention_in_backticks_is_not_reported() {
        assert_eq!(bare_mentions("the `@music` directive"), NONE);
    }

    #[test]
    fn a_mention_after_a_closed_code_span_is_reported() {
        assert_eq!(bare_mentions("`@if` and @music"), ["@music"]);
    }

    #[test]
    fn an_email_address_is_not_a_mention() {
        assert_eq!(bare_mentions("Mail noreply@anthropic.com about it"), NONE);
        assert_eq!(bare_mentions("Mail no_reply_@anthropic.com about it"), NONE);
        assert_eq!(bare_mentions("Mail 7@anthropic.com about it"), NONE);
    }

    #[test]
    fn an_at_sign_before_a_name_is_read_after_the_mention_before_it() {
        assert_eq!(bare_mentions("@alpha@beta"), ["@alpha"]);
    }

    #[test]
    fn an_at_sign_alone_or_before_a_space_or_punctuation_is_not_a_mention() {
        assert_eq!(bare_mentions("@"), NONE);
        assert_eq!(bare_mentions("meet @ noon"), NONE);
        assert_eq!(bare_mentions("@, @-x and @."), NONE);
    }

    #[test]
    fn a_name_with_a_dash_and_digits_is_returned_whole() {
        assert_eq!(bare_mentions("ask @user-42 now"), ["@user-42"]);
        assert_eq!(bare_mentions("ask @42"), ["@42"]);
    }

    #[test]
    fn a_bot_name_stops_at_its_bracket() {
        assert_eq!(bare_mentions("by @dependabot[bot]"), ["@dependabot"]);
    }
}

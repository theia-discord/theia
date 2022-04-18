//! Command help string parser

/// Parse a command help string (from a plugin config) into it's parts.
///
/// Returns a tuple of the command summary, and potentially any further
/// details on the command.
///
/// The `prefix` parameter to this function should be the _first_ prefix
/// in the bot configuration.
pub fn parse_command_help<P: AsRef<str>, C: AsRef<str>, H: AsRef<str>>(
    prefix: P,
    cmdname: C,
    helpstr: H,
) -> (String, Option<String>) {
    let cmd = format!("{}{}", prefix.as_ref(), cmdname.as_ref());
    let replacements: Vec<(String, String)> = vec![
        (String::from("%!"), cmd),
        (String::from("%PREFIX%"), format!("{}", prefix.as_ref())),
    ];

    let cmdhelp: Vec<String> = helpstr
        .as_ref()
        .trim()
        .split('\n')
        .map(|x| x.trim())
        .map(String::from)
        .map(|mut x| {
            for (pat, sub) in replacements.iter() {
                x = x.replace(pat, sub);
            }

            x
        })
        .collect();

    if cmdhelp.len() < 1 {
        return (String::new(), None);
    }

    let summary = cmdhelp[0].clone();
    let mut details: Option<String> = None;
    if cmdhelp.len() > 1 {
        details = Some(
            cmdhelp
                .iter()
                .skip(1)
                .map(String::from)
                .collect::<Vec<String>>()
                .join("\n")
                .trim()
                .to_string(),
        );
    }

    (summary, details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_helpstr() {
        let (summary, details) = parse_command_help("t;", "test", "");
        assert_eq!(String::new(), summary);
        assert_eq!(None, details);
    }

    #[test]
    fn no_details() {
        let (summary, details) = parse_command_help("t;", "test", "`%!` - test");
        assert_eq!("`t;test` - test", &summary);
        assert_eq!(None, details);
    }

    #[test]
    fn multiline_details() {
        let helpstr = r#"
            `%! [...]` - do something

            prefix `%PREFIX%`
            wheee

            paragraph
        "#;

        let (summary, details) = parse_command_help("t;", "test", helpstr);
        assert_eq!(String::from("`t;test [...]` - do something"), summary);
        assert_eq!(
            Some(String::from("prefix `t;`\nwheee\n\nparagraph")),
            details
        );
    }
}

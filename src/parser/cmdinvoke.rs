//! Command invocation parser

use ::serde::{Deserialize, Serialize};

/// A single invocation of a command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandInvocation {
    /// The command prefix used to invoke this command
    pub prefix: String,

    /// The name of the command
    pub command: String,

    /// List of flags at the start of the command invocation
    pub start_flags: Vec<String>,

    /// Command arguments, which may include flags (if the flags were
    /// surrounded by non-flags in the middle of the command)
    pub arguments: Vec<String>,

    /// List of flags at the end of the command invocation
    pub end_flags: Vec<String>,
}

impl CommandInvocation {
    /// Checks whether the `-help` flag was passed
    pub fn help_requested(&self) -> bool {
        self.has_startflag_any(&["-help"])
    }

    /// Checks whether any of the given flags were present in `start_flags`
    pub fn has_startflag_any<T: AsRef<str>>(&self, flags: &[T]) -> bool {
        self.start_flags
            .iter()
            .any(|e| flags.iter().any(|f| f.as_ref() == e))
    }

    /// Checks whether any of the given flags were present in `end_flags`
    pub fn has_endflag_any<T: AsRef<str>>(&self, flags: &[T]) -> bool {
        self.end_flags
            .iter()
            .any(|e| flags.iter().any(|f| f.as_ref() == e))
    }

    /// Checks whether any of the given flags were present in `arguments`
    pub fn has_argument_any<T: AsRef<str>>(&self, flags: &[T]) -> bool {
        self.arguments
            .iter()
            .any(|e| flags.iter().any(|f| f.as_ref() == e))
    }
}

impl CommandInvocation {
    /// Parse a command invocation.
    ///
    /// Takes a list of command prefixes, and an input string. A command
    /// invocation can use any of the specified command prefixes.
    pub fn parse<P: AsRef<str>, M: AsRef<str>>(prefixes: &[P], input: M) -> Option<Self> {
        let input = String::from(input.as_ref());

        // Try to match from our list of prefixes

        let mut matched_prefix: Option<String> = None;
        let mut prefixes: Vec<String> = prefixes
            .iter()
            .map(|x| x.as_ref())
            .map(String::from)
            .collect();
        prefixes.reverse();
        'prefix_match: for prefix in prefixes.into_iter() {
            if input.starts_with(&prefix) {
                matched_prefix = Some(prefix);
                break 'prefix_match;
            }
        }

        if matched_prefix.is_none() {
            return None;
        }

        // Okay, we have a known prefix, let's split this thing apart

        let matched_prefix = matched_prefix.unwrap();
        let mut input = String::from(input);
        input.replace_range(..matched_prefix.len(), "");

        let argsplit: Vec<String> = input.split_whitespace().map(String::from).collect();
        if argsplit.len() < 1 {
            return None;
        }

        let mut args: Vec<String> = Vec::new();
        let mut argsplit_idx = 1;

        while argsplit_idx < argsplit.len() {
            let mut arg = argsplit[argsplit_idx].clone();
            if arg.starts_with('"') {
                while !arg.ends_with('"') {
                    argsplit_idx += 1;
                    arg = format!("{} {}", arg, argsplit[argsplit_idx]);
                }

                arg = arg
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string();
            }

            args.push(arg);
            argsplit_idx += 1;
        }

        // Split `args` into start flags, arguments, and end flags

        let mut start_flags: Vec<String> = Vec::new();
        let mut arguments: Vec<String> = Vec::new();
        let mut end_flags: Vec<String> = Vec::new();

        let mut fp_first_arg: Option<usize> = None;
        let mut fp_first_endflag: Option<usize> = None;

        'fp_passone: for argidx in 0..args.len() {
            if fp_first_arg.is_none() && !args[argidx].starts_with('-') {
                fp_first_arg = Some(argidx);
                break 'fp_passone;
            }
        }

        'fp_passtwo: for argidx in (0..args.len()).rev() {
            if fp_first_endflag.is_none() && !args[argidx].starts_with('-') {
                fp_first_endflag = Some(argidx + 1);
                break 'fp_passtwo;
            }
        }

        for argidx in 0..args.len() {
            if fp_first_endflag.is_some() && fp_first_endflag.unwrap() <= argidx {
                end_flags.push(args[argidx].clone());
            } else if fp_first_arg.is_some() && fp_first_arg.unwrap() <= argidx {
                arguments.push(args[argidx].clone());
            } else {
                start_flags.push(args[argidx].clone());
            }
        }

        // Return the CommandInvocation object
        Some(CommandInvocation {
            prefix: matched_prefix.clone(),
            command: argsplit[0]
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string(),
            start_flags,
            arguments,
            end_flags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_prefix_returns_none() {
        assert_eq!(
            None,
            CommandInvocation::parse(&["t;", "t!"], "unprefixed -test",)
        );
    }

    #[test]
    fn mismatched_prefix_returns_none() {
        assert_eq!(
            None,
            CommandInvocation::parse(&["t;", "t!"], "tx;unprefixed -test",)
        );
    }

    #[test]
    fn prefix_selection_correct() {
        let cmd_one = CommandInvocation::parse(&["t;", "t!"], "t;test").unwrap();
        assert_eq!("t;", &cmd_one.prefix);

        let cmd_two = CommandInvocation::parse(&["t;", "t!"], "t!test").unwrap();
        assert_eq!("t!", &cmd_two.prefix);
    }

    #[test]
    fn flag_positions_correct() {
        let cmd = CommandInvocation::parse(
            &["t;", "t!"],
            "t;flagtest -startone -starttwo test -midone test -endone -endtwo",
        )
        .unwrap();

        assert!(cmd.has_startflag_any(&["-startone"]));
        assert!(cmd.has_startflag_any(&["-starttwo"]));
        assert!(cmd.has_argument_any(&["-midone"]));
        assert!(cmd.has_endflag_any(&["-endone"]));
        assert!(cmd.has_endflag_any(&["-endtwo"]));
    }
}

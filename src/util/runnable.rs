//! Serde-compatible field for an executable + arguments

use ::serde::{de, Deserialize};
use ::std::fmt::{self, Formatter};

/// Serde-compatible field for an executable + arguments
#[derive(Debug, Clone)]
pub struct RunnableCommand {
    pub command: String,
    pub arguments: Vec<String>,
}

impl RunnableCommand {
    pub fn as_tokio_command(&self) -> ::tokio::process::Command {
        let mut cmd = ::tokio::process::Command::new(&self.command);
        cmd.args(&self.arguments);
        cmd
    }
}

impl<'de> Deserialize<'de> for RunnableCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct RunnableCommandVisitor;

        impl<'de> de::Visitor<'de> for RunnableCommandVisitor {
            type Value = RunnableCommand;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "RunnableCommand")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'de>,
            {
                if let Some(command) = seq.next_element::<String>()? {
                    let mut arguments: Vec<String> = Vec::new();
                    while let Some(arg) = seq.next_element::<String>()? {
                        arguments.push(arg);
                    }

                    return Ok(Self::Value { command, arguments });
                }

                Err(de::Error::invalid_value(de::Unexpected::Seq, &self))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if s.len() > 0 {
                    return Ok(Self::Value {
                        command: String::from("sh"),
                        arguments: vec![String::from("-c"), String::from(s)],
                    });
                }

                Err(de::Error::invalid_value(de::Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_map(RunnableCommandVisitor {})
    }
}

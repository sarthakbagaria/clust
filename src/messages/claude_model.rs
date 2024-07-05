use crate::macros::impl_enum_string_serialization;
use std::fmt::Display;

/// The model that will complete your prompt.
///
/// See [models](https://docs.anthropic.com/claude/docs/models-overview) for additional details and options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClaudeModel {
    // Claude 3 Opus
    /// Claude 3 Opus at 2024/02/29.
    Claude3Opus20240229,
    // Claude 3 Sonnet
    /// Claude 3 Sonnet at 2024/02/29.
    Claude3Sonnet20240229,
    // Claude 3 Haiku
    /// Claude 3 Haiku at 2024/03/07.
    Claude3Haiku20240307,
    // Claude 3.5 Sonnet
    /// Claude 3.5 Sonnet at 2024/06/20
    Claude35Sonnet20240620,
}

impl Default for ClaudeModel {
    fn default() -> Self {
        Self::Claude3Sonnet20240229
    }
}

impl Display for ClaudeModel {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            | ClaudeModel::Claude3Opus20240229 => {
                write!(f, "claude-3-opus-20240229")
            },
            | ClaudeModel::Claude3Sonnet20240229 => {
                write!(f, "claude-3-sonnet-20240229")
            },
            | ClaudeModel::Claude3Haiku20240307 => {
                write!(f, "claude-3-haiku-20240307")
            },
            | ClaudeModel::Claude35Sonnet20240620 => {
                write!(f, "claude-3-5-sonnet-20240620")
            },
        }
    }
}

impl ClaudeModel {
    pub(crate) fn max_tokens(&self) -> u32 {
        match self {
            | ClaudeModel::Claude3Opus20240229 => 4096,
            | ClaudeModel::Claude3Sonnet20240229 => 4096,
            | ClaudeModel::Claude3Haiku20240307 => 4096,
            | ClaudeModel::Claude35Sonnet20240620 => 4096,
        }
    }
}

impl_enum_string_serialization!(
    ClaudeModel,
    Claude3Opus20240229 => "claude-3-opus-20240229",
    Claude3Sonnet20240229 => "claude-3-sonnet-20240229",
    Claude3Haiku20240307 => "claude-3-haiku-20240307",
    Claude35Sonnet20240620 => "claude-3-5-sonnet-20240620"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(
            ClaudeModel::default(),
            ClaudeModel::Claude3Sonnet20240229
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            ClaudeModel::Claude3Opus20240229.to_string(),
            "claude-3-opus-20240229"
        );
        assert_eq!(
            ClaudeModel::Claude3Sonnet20240229.to_string(),
            "claude-3-sonnet-20240229"
        );
        assert_eq!(
            ClaudeModel::Claude3Haiku20240307.to_string(),
            "claude-3-haiku-20240307"
        );
        assert_eq!(
            ClaudeModel::Claude35Sonnet20240620.to_string(),
            "claude-3-5-sonnet-20240620"
        );
    }

    #[test]
    fn max_tokens() {
        assert_eq!(
            ClaudeModel::Claude3Opus20240229.max_tokens(),
            4096
        );
        assert_eq!(
            ClaudeModel::Claude3Sonnet20240229.max_tokens(),
            4096
        );
        assert_eq!(
            ClaudeModel::Claude3Haiku20240307.max_tokens(),
            4096
        );
        assert_eq!(
            ClaudeModel::Claude35Sonnet20240620.max_tokens(),
            4096
        );
    }

    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::from_str::<ClaudeModel>("\"claude-3-opus-20240229\"")
                .unwrap(),
            ClaudeModel::Claude3Opus20240229
        );
        assert_eq!(
            serde_json::from_str::<ClaudeModel>("\"claude-3-sonnet-20240229\"")
                .unwrap(),
            ClaudeModel::Claude3Sonnet20240229
        );
        assert_eq!(
            serde_json::from_str::<ClaudeModel>("\"claude-3-haiku-20240307\"")
                .unwrap(),
            ClaudeModel::Claude3Haiku20240307
        );
        assert_eq!(
            serde_json::from_str::<ClaudeModel>(
                "\"claude-3-5-sonnet-20240620\""
            )
            .unwrap(),
            ClaudeModel::Claude35Sonnet20240620
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::to_string(&ClaudeModel::Claude3Opus20240229).unwrap(),
            "\"claude-3-opus-20240229\""
        );
        assert_eq!(
            serde_json::to_string(&ClaudeModel::Claude3Sonnet20240229).unwrap(),
            "\"claude-3-sonnet-20240229\""
        );
        assert_eq!(
            serde_json::to_string(&ClaudeModel::Claude3Haiku20240307).unwrap(),
            "\"claude-3-haiku-20240307\""
        );
        assert_eq!(
            serde_json::to_string(&ClaudeModel::Claude35Sonnet20240620)
                .unwrap(),
            "\"claude-3-5-sonnet-20240620\""
        );
    }
}

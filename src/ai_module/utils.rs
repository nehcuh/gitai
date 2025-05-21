use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Represents a chat message with a role and content
///
/// This structure is used for both requests to and responses from AI chat models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64, // Typically a UNIX Timestamp
    pub model: String,
    pub system_fingerprint: Option<String>, // This field exists based on the exampale provided
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
    // pub logprobs: Option<serde_json::Value> // If logprobs parsing is needed
}

// Removes <think>...</think> tags and their content from a given string
//
// The regex pattern is compiled once using lazy_static for better performance
// since this function might be called frequently.
lazy_static! {
    static ref RE_THINK_TAGS: Regex = Regex::new(r"(?s)<think>.*?</think>").unwrap();
}

pub fn clean_ai_output(text: &str) -> String {
    // Using the pre-compiled regex pattern for better perfomance
    RE_THINK_TAGS.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_clean_ai_output_no_tags() {
        let input = "This is a normal commit message.";
        let expected = "This is a normal commit message.";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_single_tag() {
        let input = "<think>This is a thought.</think>Actual commit message.";
        let expected = "Actual commit message.";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_multiple_tags() {
        let input = "<think>Thought 1</think>Commit part 1. <think>Thought 2</think>Commit part 2.";
        let expected = "Commit part 1. Commit part 2.";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_multiline_tag() {
        let input =
            "Commit start.\n<think>\nMultiline thought\nAnother line\n</think>\nCommit end.";
        let expected = "Commit start.\n\nCommit end."; // Regex replace_all removes the tag and its content
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_empty_tag() {
        let input = "Before<think></think>After";
        let expected = "BeforeAfter";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_tag_at_start() {
        let input = "<think>Initial thought.</think>The rest of the message.";
        let expected = "The rest of the message.";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_tag_at_end() {
        let input = "Message part.<think>Final thought.</think>";
        let expected = "Message part.";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_only_tag() {
        let input = "<think>This is entirely a thought.</think>";
        let expected = "";
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_nested_tags_not_specifically_handled_outermost_wins() {
        // Standard regex (non-recursive) will match the shortest non-greedy .*?
        // For input "<think>outer<think>inner</think>still outer</think>Commit message."
        // the regex matches and removes "<think>outer<think>inner</think>",
        // leaving "still outer</think>Commit message.".
        let input = "<think>outer<think>inner</think>still outer</think>Commit message.";
        let expected = "still outer</think>Commit message.";
        assert_eq!(clean_ai_output(input), expected);

        let input_greedy_would_fail = "<think>thought 1</think> message <think>thought 2</think>";
        // if it were greedy like <think>.*</think>, it would consume " message "
        // but with .*? it correctly separates them.
        let re_greedy_test = Regex::new(r"(?s)<think>.*</think>").unwrap();
        assert_ne!(
            re_greedy_test
                .replace_all(input_greedy_would_fail, "")
                .into_owned(),
            " message "
        );
    }

    #[test]
    fn test_clean_ai_output_no_closing_tag_leaves_untouched_if_regex_is_strict() {
        // Our regex r"(?s)<think>.*?</think>" requires a closing tag.
        // If it's not found, it shouldn't match.
        let input = "<think>This thought is not closed. Actual message.";
        let expected = "<think>This thought is not closed. Actual message."; // Or however your regex behaves with unclosed tags
        assert_eq!(clean_ai_output(input), expected);
    }

    #[test]
    fn test_clean_ai_output_tags_with_attributes_ignored_by_current_regex() {
        // The current regex <think> is simple and doesn't account for <think foo="bar">
        // This test confirms it only removes simple <think> tags.
        let input = "<think foo=\"bar\">A thought with attributes.</think>Commit message.";
        let expected = "<think foo=\"bar\">A thought with attributes.</think>Commit message."; // Current regex won't match this
        assert_eq!(clean_ai_output(input), expected);

        let input_simple = "<think>Simple thought.</think>Commit message.";
        let expected_simple = "Commit message.";
        assert_eq!(clean_ai_output(input_simple), expected_simple);
    }
    #[test]
    fn test_complex_scenario_with_varied_spacing_and_content() {
        let input = "  <think>  Leading space thought. </think> Commit part 1.   <think>\\nMultiline\\n  Thought\\n</think>Middle part.<think>Trailing thought</think>   Final part.  ";
        let expected = "   Commit part 1.   Middle part.   Final part.  ";
        // clean_ai_output itself does not trim the surrounding whitespace from the overall string.
        // That kind of trimming might happen at a later stage if desired (e.g., before committing).
        assert_eq!(clean_ai_output(input), expected);
    }
}

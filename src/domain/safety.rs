use lazy_static::lazy_static;
use regex::Regex;

/// Detect common AI safety refusal patterns across major LLM providers.
pub fn is_safety_refusal(text: &str) -> bool {
    let text = text.to_lowercase();
    text.contains("i cannot assist")
        || text.contains("i am unable to")
        || text.contains("my safety guidelines")
        || text.contains("i can't fulfill this request")
        || text.contains("i'm sorry, but")
        || text.contains("i must decline")
        || text.contains("as an ai")
        || text.contains("i can't help with")
        || text.contains("i'm not able to")
        || text.contains("against my guidelines")
        || text.contains("i cannot provide")
        || text.contains("i can't provide")
        || text.contains("i cannot help")
        || text.contains("potentially harmful")
        || text.contains("violates my")
        || text.contains("i'm unable to assist")
}

lazy_static! {
    static ref SHELL_METACHART: Regex = Regex::new(r#"[;&|><$()`]"#).unwrap();
}

/// Sanitize arguments to prevent shell injection.
/// Even though we use direct exec, we want to prevent AI from trying to escape the context.
pub fn sanitize_custom_arg(arg: &str) -> bool {
    !SHELL_METACHART.is_match(arg)
}

/// Clean an argument by removing potentially dangerous characters if they exist,
/// but in our case we prefer to reject the argument entirely for safety.
pub fn is_clean_argument(args: &[String]) -> bool {
    args.iter().all(|arg| sanitize_custom_arg(arg))
}

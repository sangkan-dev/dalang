/// Detect if the LLM response is a safety refusal
pub fn is_safety_refusal(response: &str) -> bool {
    let lower = response.to_lowercase();

    // Standard refusal phrases
    let phrases = [
        "i cannot assist",
        "i can't assist",
        "i'm sorry, but i can't",
        "i am sorry, but i can't",
        "i cannot fulfill",
        "i can't fulfill",
        "policy against",
        "against my safety",
        "it is not appropriate",
        "i am not able to",
        "i'm not able to",
        "as an ai language model",
    ];

    for phrase in phrases {
        if lower.contains(phrase) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safety_refusal() {
        assert!(is_safety_refusal(
            "I'm sorry, but I can't assist with that request. It involves hacking."
        ));
        assert!(is_safety_refusal(
            "As an AI language model, I cannot fulfill this request."
        ));
        assert!(!is_safety_refusal(
            "Here is the result of your scan: Port 80 is open."
        ));
    }
}

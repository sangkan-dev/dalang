/// Maximum bytes for a single tool observation injected into LLM context.
/// Keeps first + last portions with a truncation notice in the middle.
pub const MAX_OBSERVATION_BYTES: usize = 12_000;

/// Rough token budget: leave headroom below model context limit.
/// Most models support 128k tokens; we aim to stay under ~100k tokens
/// to leave room for the LLM response.
const TOKEN_BUDGET: usize = 100_000;

/// Rough estimate: 1 token ≈ 4 characters for English text.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Truncate tool output that exceeds the byte limit.
/// Keeps the first and last portions with a notice in the middle.
pub fn truncate_output(output: &str, max_bytes: usize) -> String {
    if output.len() <= max_bytes {
        return output.to_string();
    }
    let keep = max_bytes / 2;
    let head = &output[..keep];
    let tail = &output[output.len() - keep..];
    let original_lines = output.lines().count();
    let truncated_bytes = output.len() - max_bytes;
    format!(
        "{}\n\n... [TRUNCATED: {} bytes / ~{} lines omitted — output too large for context window] ...\n\n{}",
        head, truncated_bytes, original_lines, tail
    )
}

pub struct ContextManager {
    memory: Vec<String>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }

    /// Restore a ContextManager from previously saved observations.
    pub fn from_observations(obs: Vec<String>) -> Self {
        let mut cm = Self::new();
        // Only keep the last 20
        let start = if obs.len() > 20 { obs.len() - 20 } else { 0 };
        cm.memory = obs[start..].to_vec();
        cm
    }

    /// Read-only access to the observation list (for persistence).
    pub fn observations(&self) -> &[String] {
        &self.memory
    }

    pub fn add_observation(&mut self, observation: String) {
        // Retain last 20 significant observations for multi-stage pentests
        if self.memory.len() >= 20 {
            self.memory.remove(0);
        }
        self.memory.push(observation);
    }

    pub fn get_summary_prompt(&self) -> String {
        if self.memory.is_empty() {
            return String::from("No previous observations recorded in persistent memory.");
        }

        let mut prompt = String::from(
            "### PERSISTENT CONTEXT MEMORY (Last observations):\n\
            Reference these observations to avoid repeating work. Note specific URLs, parameters, \n\
            and findings from previous tool executions when planning next steps.\n",
        );
        for (idx, obs) in self.memory.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", idx + 1, obs));
        }
        prompt
    }
}

/// Compact the message history when it exceeds the token budget.
///
/// Strategy: keep the system prompt (first message) and the last N messages intact.
/// Middle messages (old observations/responses) are replaced with a single compact summary.
pub fn compact_messages(messages: &mut Vec<crate::domain::models::Message>) {
    let total_tokens: usize = messages.iter().map(|m| estimate_tokens(&m.content)).sum();

    if total_tokens <= TOKEN_BUDGET {
        return;
    }

    // We need to cut. Keep system prompt (idx 0) and last 4 messages.
    let keep_tail = 4.min(messages.len().saturating_sub(1));
    if messages.len() <= keep_tail + 1 {
        // Not enough messages to compact — truncate individual large messages instead
        for msg in messages.iter_mut() {
            if msg.role == "user" && estimate_tokens(&msg.content) > 3000 {
                msg.content = truncate_output(&msg.content, MAX_OBSERVATION_BYTES);
            }
        }
        return;
    }

    let middle_start = 1;
    let middle_end = messages.len() - keep_tail;

    // Build a compact summary of the middle messages
    let mut summary_parts: Vec<String> = Vec::new();
    for msg in &messages[middle_start..middle_end] {
        let role = &msg.role;
        let content = &msg.content;

        if role == "user" && content.contains("OBSERVATION FROM") {
            // Extract skill name and line count from observation
            if let Some(skill_start) = content.find('`')
                && let Some(skill_end) = content[skill_start + 1..].find('`') {
                    let skill = &content[skill_start + 1..skill_start + 1 + skill_end];
                    let lines = content.lines().count();
                    summary_parts
                        .push(format!("- Executed `{}`: {} lines of output", skill, lines));
                    continue;
                }
            let lines = content.lines().count();
            summary_parts.push(format!("- Tool observation: {} lines", lines));
        } else if role == "assistant" && content.len() > 200 {
            // Keep first 200 chars of assistant reasoning
            summary_parts.push(format!("- AI reasoning: {}...", &content[..200]));
        } else if role == "assistant" {
            summary_parts.push(format!("- AI: {}", content));
        }
        // Skip system re-prompt messages (they're boilerplate)
    }

    let compact = format!(
        "### COMPACTED CONTEXT (iterations 1-{})\n\
        The following is a summary of previous iterations that were compacted to save context space:\n\
        {}\n\n\
        Continue the audit based on these previous findings. Do NOT repeat tools that were already executed above.",
        middle_end - 1,
        summary_parts.join("\n")
    );

    // Remove the middle messages and insert the compact summary
    let tail: Vec<crate::domain::models::Message> = messages[middle_end..].to_vec();
    messages.truncate(1); // Keep system prompt
    messages.push(crate::domain::models::Message::user(&compact));
    messages.extend(tail);
}

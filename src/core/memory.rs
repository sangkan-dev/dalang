pub struct ContextManager {
    memory: Vec<String>,
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

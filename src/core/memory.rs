pub struct ContextManager {
    memory: Vec<String>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
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

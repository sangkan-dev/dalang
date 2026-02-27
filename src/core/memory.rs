pub struct ContextManager {
    memory: Vec<String>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }

    pub fn add_observation(&mut self, observation: String) {
        // We limit memory to last 5 significant observations for now
        if self.memory.len() >= 5 {
            self.memory.remove(0);
        }
        self.memory.push(observation);
    }

    pub fn get_summary_prompt(&self) -> String {
        if self.memory.is_empty() {
            return String::from("No previous observations recorded in persistent memory.");
        }

        let mut prompt = String::from("### PERSISTENT CONTEXT MEMORY (Last observations):\n");
        for (idx, obs) in self.memory.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", idx + 1, obs));
        }
        prompt
    }
}

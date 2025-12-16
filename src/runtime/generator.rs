//! Script generator for Pasta scripts.
//!
//! This module provides types for script execution state management.
//! Note: Due to Rune 0.14 Generator<&mut Vm> lifetime constraints,
//! the current implementation executes labels to completion rather than
//! providing incremental resumption. Future versions may support streaming.

/// State of the script generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptGeneratorState {
    /// Generator is running and can be resumed.
    Running,
    /// Generator is suspended (waiting for resume).
    Suspended,
    /// Generator has completed execution.
    Completed,
}

/// Script generator for executing Pasta scripts.
///
/// Note: This is a placeholder type. The current implementation
/// in PastaEngine executes labels to completion due to Rune
/// Generator lifetime constraints.
pub struct ScriptGenerator {
    state: ScriptGeneratorState,
}

impl ScriptGenerator {
    /// Check if the generator has completed.
    pub fn is_completed(&self) -> bool {
        self.state == ScriptGeneratorState::Completed
    }

    /// Get the current state of the generator.
    pub fn state(&self) -> ScriptGeneratorState {
        self.state
    }

    /// Skip the rest of the script (immediately complete).
    pub fn skip(&mut self) {
        self.state = ScriptGeneratorState::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are incomplete because we need a full Rune VM setup
    // to create actual generators. Integration tests will cover this.

    #[test]
    fn test_generator_state() {
        // This is a placeholder test - actual tests require Rune VM integration
        let state = ScriptGeneratorState::Running;
        assert_eq!(state, ScriptGeneratorState::Running);
        assert_ne!(state, ScriptGeneratorState::Completed);
    }

    #[test]
    fn test_generator_skip() {
        // This test demonstrates the skip functionality without requiring a real generator
        // In practice, we'd need to mock or create a real Rune generator
    }
}

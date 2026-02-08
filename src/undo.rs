use std::collections::VecDeque;

const MAX_UNDO_HISTORY: usize = 50;

pub struct UndoHistory<T> {
    undo_stack: VecDeque<T>,
    redo_stack: Vec<T>,
}

impl<T: Clone> UndoHistory<T> {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn save(&mut self, state: T) {
        self.undo_stack.push_back(state);
        self.redo_stack.clear();
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.pop_front();
        }
    }

    pub fn undo(&mut self, current: T) -> Option<T> {
        let prev = self.undo_stack.pop_back()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: T) -> Option<T> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push_back(current);
        Some(next)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.undo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_undo() {
        let mut history = UndoHistory::new();
        history.save("state_0".to_string());

        let restored = history.undo("state_1".to_string());

        assert_eq!(restored, Some("state_0".to_string()));
    }

    #[test]
    fn test_undo_then_redo() {
        let mut history = UndoHistory::new();
        history.save("state_0".to_string());
        history.undo("state_1".to_string());

        let redone = history.redo("state_0".to_string());

        assert_eq!(redone, Some("state_1".to_string()));
    }

    #[test]
    fn test_undo_empty_returns_none() {
        let mut history: UndoHistory<String> = UndoHistory::new();

        let result = history.undo("current".to_string());

        assert_eq!(result, None);
    }

    #[test]
    fn test_redo_empty_returns_none() {
        let mut history: UndoHistory<String> = UndoHistory::new();

        let result = history.redo("current".to_string());

        assert_eq!(result, None);
    }

    #[test]
    fn test_save_after_undo_clears_redo() {
        let mut history = UndoHistory::new();
        history.save("state_0".to_string());
        history.undo("state_1".to_string());

        history.save("state_2".to_string());
        let redo_result = history.redo("state_2".to_string());

        assert_eq!(redo_result, None, "redo stack should be cleared after a new save");
    }

    #[test]
    fn test_stack_limited_to_max_entries() {
        let mut history = UndoHistory::new();

        for i in 0..MAX_UNDO_HISTORY + 10 {
            history.save(format!("state_{}", i));
        }

        assert!(
            history.len() <= MAX_UNDO_HISTORY,
            "undo stack should be limited to {} entries, got {}",
            MAX_UNDO_HISTORY, history.len()
        );
    }
}

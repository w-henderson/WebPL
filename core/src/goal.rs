use crate::HeapTermPtr;

pub struct Goal(pub HeapTermPtr, pub Option<GoalPtr>);

pub type GoalPtr = usize;

#[derive(Default)]
pub struct GoalArena {
    arena: Vec<Goal>,
}

pub struct Checkpoint(usize);

impl GoalArena {
    pub fn alloc(&mut self, term: HeapTermPtr, prev: Option<GoalPtr>) -> GoalPtr {
        let result = self.arena.len();
        self.arena.push(Goal(term, prev));
        result
    }

    pub fn get(&self, ptr: GoalPtr) -> &Goal {
        &self.arena[ptr]
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.arena.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.arena.truncate(checkpoint.0);
    }
}

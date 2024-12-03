use crate::HeapTermPtr;

#[derive(Debug)]
pub struct Goal(HeapTermPtr, Option<GoalPtr>);

pub type GoalPtr = usize;

#[derive(Default, Debug)]
pub struct Goals {
    current: Option<GoalPtr>,
    goals: Vec<Goal>,
}

pub struct Checkpoint(Option<GoalPtr>, usize);

impl Goals {
    pub fn new(query: &[HeapTermPtr]) -> Self {
        let mut goals = Self::default();
        let mut goal = None;

        for term in query.iter().rev() {
            goal = Some(goals.alloc(*term, goal));
        }

        goals.current = goal;

        goals
    }

    pub fn alloc(&mut self, term: HeapTermPtr, prev: Option<GoalPtr>) -> GoalPtr {
        let result = self.goals.len();
        self.goals.push(Goal(term, prev));
        result
    }

    pub fn current(&self) -> Option<GoalPtr> {
        self.current.map(|ptr| self.goals[ptr].term())
    }

    pub fn pop(&mut self) {
        if let Some(ptr) = self.current.take() {
            self.current = self.goals[ptr].prev_ptr();
        }
    }

    pub fn push(&mut self, term: HeapTermPtr) {
        let prev = self.current.take();
        self.current = Some(self.alloc(term, prev));
    }

    pub fn is_complete(&self) -> bool {
        self.current.is_none()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.current, self.goals.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.current = checkpoint.0;
        self.goals.truncate(checkpoint.1);
    }
}

impl Goal {
    pub fn term(&self) -> HeapTermPtr {
        self.0
    }

    pub fn prev_ptr(&self) -> Option<GoalPtr> {
        self.1
    }
}

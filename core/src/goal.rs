use crate::gc::GCRewritable;
use crate::HeapTermPtr;

pub struct Goal(pub(crate) HeapTermPtr, pub(crate) Option<GoalPtr>);

pub type GoalPtr = usize;

#[derive(Default)]
pub struct Goals {
    current: Option<GoalPtr>,
    goals: Vec<Goal>,
}

#[derive(Clone, Copy)]
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

    pub fn current(&self) -> Option<HeapTermPtr> {
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

    pub fn iter(&self) -> GoalIterator {
        GoalIterator {
            goals: self,
            current: self.current,
        }
    }

    pub fn iter_from(&self, checkpoint: Checkpoint) -> GoalIterator {
        GoalIterator {
            goals: self,
            current: checkpoint.0,
        }
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

impl GCRewritable for Goals {
    fn rewrite(&mut self, map: &[usize], _: &[usize]) {
        for goal in self.goals.iter_mut() {
            if let Some(ptr) = map.get(goal.0) {
                // This will always be true for live goals
                // TODO: collect dead goals
                goal.0 = *ptr;
            }
        }
    }
}

pub struct GoalIterator<'a> {
    goals: &'a Goals,
    current: Option<GoalPtr>,
}

impl<'a> Iterator for GoalIterator<'a> {
    type Item = HeapTermPtr;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.current {
            let Goal(heap_term_ptr, next) = &self.goals.goals[ptr];
            self.current = *next;
            Some(*heap_term_ptr)
        } else {
            None
        }
    }
}

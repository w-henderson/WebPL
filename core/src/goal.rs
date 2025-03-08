use crate::gc::GCRewritable;
use crate::HeapTermPtr;

#[derive(Clone, Copy, Debug)]
pub struct Goal(pub(crate) HeapTermPtr, pub(crate) Option<GoalPtr>);

pub type GoalPtr = usize;

#[derive(Default, Debug)]
pub struct Goals {
    pub(crate) current: Option<GoalPtr>,
    pub(crate) goals: Vec<Goal>,
    pub(crate) pending: Option<HeapTermPtr>,
}

#[derive(Clone, Copy)]
pub struct Checkpoint(pub(crate) Option<GoalPtr>, pub(crate) usize);

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

    pub fn pop(&mut self, determinate: bool) {
        if let Some(ptr) = self.current.take() {
            self.current = self.goals[ptr].prev_ptr();
            if determinate && ptr == self.goals.len() - 1 {
                self.goals.pop();
            }
        }

        if let Some(pending) = self.pending.take() {
            self.push(pending);
        }
    }

    pub fn push(&mut self, term: HeapTermPtr) {
        let prev = self.current.take();
        self.current = Some(self.alloc(term, prev));
    }

    pub fn push_pending(&mut self, term: HeapTermPtr) {
        self.pending = Some(term);
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
    fn rewrite(&mut self, from: usize, map: &[usize], _: &[usize]) {
        for Goal(term, _) in self.goals.iter_mut().skip(from) {
            *term = map[*term];
        }
    }
}

pub struct GoalIterator<'a> {
    goals: &'a Goals,
    current: Option<GoalPtr>,
}

impl Iterator for GoalIterator<'_> {
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

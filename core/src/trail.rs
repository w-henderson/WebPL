use crate::heap::Heap;
use crate::HeapTermPtr;

#[derive(Default)]
pub struct Trail {
    vars: Vec<HeapTermPtr>,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(usize);

impl Trail {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, var: HeapTermPtr) {
        self.vars.push(var);
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.vars.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint, vars: &mut Heap) {
        for var in (checkpoint.0..self.vars.len()).rev() {
            vars.unbind(self.vars[var]);
        }

        self.vars.truncate(checkpoint.0);
    }
}

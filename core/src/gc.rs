use crate::goal::Goals;
use crate::heap::Heap;
use crate::trail::Trail;
use crate::{ChoicePoint, HeapTerm, HeapTermPtr, Solver};

pub struct GarbageCollector {
    map: Vec<usize>,
    scheduler: GCScheduler,
}

pub struct GCScheduler {
    absolute_threshold: usize,
    relative_threshold: f64,
    cooldown: usize,
    remaining_cooldown: usize,
}

pub trait GCRewritable {
    fn rewrite(&mut self, map: &[usize]);
}

impl GarbageCollector {
    pub fn new(absolute_threshold: usize, relative_threshold: f64, cooldown: usize) -> Self {
        Self {
            map: Vec::new(),
            scheduler: GCScheduler {
                absolute_threshold,
                relative_threshold,
                cooldown,
                remaining_cooldown: 0,
            },
        }
    }

    pub fn run(solver: &mut Solver) {
        let roots = solver.gc.get_roots(
            &solver.var_map,
            &solver.goals,
            &solver.choice_points,
            &solver.trail,
        );
        solver.gc.collect(&mut solver.heap, roots);
        solver.gc.rewrite(
            &mut solver.var_map,
            &mut solver.goals,
            &mut solver.choice_points,
            &mut solver.trail,
        );
    }

    pub fn get_roots<'a>(
        &self,
        vars: &'a [(String, usize)],
        goals: &'a Goals,
        choice_points: &'a [ChoicePoint],
        trail: &'a Trail,
    ) -> impl Iterator<Item = HeapTermPtr> + 'a {
        vars.iter()
            .map(|(_, ptr)| *ptr)
            .chain(goals.iter())
            .chain(choice_points.iter().flat_map(|cp| {
                goals
                    .iter_from(cp.goals_checkpoint)
                    .chain(std::iter::once(cp.heap_checkpoint.0))
            }))
            .chain(trail.iter())
    }

    pub fn collect(&mut self, heap: &mut Heap, roots: impl Iterator<Item = HeapTermPtr>) {
        self.map.clear();
        self.map.resize(heap.data.len(), 0);

        for root in roots {
            self.mark(heap, root);
        }

        self.compact(heap);
    }

    fn mark(&mut self, heap: &Heap, ptr: HeapTermPtr) {
        if self.map[ptr] != 0 {
            return;
        }

        self.map[ptr] = 1;

        match &heap.data[ptr] {
            HeapTerm::Var(ptr) => self.mark(heap, *ptr),
            HeapTerm::Compound(_, _, Some(next)) => self.mark(heap, *next),
            HeapTerm::CompoundCons(v, next) => {
                self.mark(heap, *v);
                if let Some(next) = next {
                    self.mark(heap, *next);
                }
            }
            _ => (),
        }
    }

    fn compact(&mut self, heap: &mut Heap) {
        let mut new_ptr: HeapTermPtr = 0;

        // Shuffle data down the heap, overwriting dead data
        for (old_ptr, ptr) in self
            .map
            .iter_mut()
            .enumerate()
            .filter(|(_, ptr)| **ptr != 0)
        {
            *ptr = new_ptr;
            new_ptr += 1;

            if *ptr != old_ptr {
                heap.data[*ptr] = heap.data[old_ptr];
            }
        }

        heap.data.truncate(new_ptr);

        // Rewrite internal pointers
        for term in heap.data.iter_mut() {
            match term {
                HeapTerm::Var(ptr) => *ptr = self.map[*ptr],
                HeapTerm::Compound(_, _, Some(next)) => *next = self.map[*next],
                HeapTerm::CompoundCons(head, tail) => {
                    *head = self.map[*head];
                    if let Some(tail) = tail {
                        *tail = self.map[*tail];
                    }
                }
                _ => (),
            }
        }
    }

    pub fn rewrite(
        &self,
        vars: &mut [(String, usize)],
        goals: &mut Goals,
        choice_points: &mut [ChoicePoint],
        trail: &mut Trail,
    ) {
        vars.rewrite(&self.map);
        goals.rewrite(&self.map);
        choice_points.rewrite(&self.map);
        trail.rewrite(&self.map);
    }

    pub fn should_run(&mut self, heap: &Heap) -> bool {
        self.scheduler.should_run(heap)
    }
}

impl GCRewritable for [(String, usize)] {
    fn rewrite(&mut self, map: &[usize]) {
        for (_, ptr) in self.iter_mut() {
            *ptr = map[*ptr];
        }
    }
}

impl GCScheduler {
    fn should_run(&mut self, heap: &Heap) -> bool {
        let should_run = heap.data.len() > self.absolute_threshold
            && (heap.size() as f64) / (heap.capacity() as f64) > self.relative_threshold
            && self.remaining_cooldown == 0;

        if should_run {
            self.remaining_cooldown = self.cooldown;
        } else {
            self.remaining_cooldown = self.remaining_cooldown.saturating_sub(1);
        }

        should_run
    }
}

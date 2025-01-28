use crate::goal::Goals;
use crate::heap::Heap;
use crate::trail::Trail;
use crate::{ChoicePoint, HeapTerm, HeapTermPtr, Solver};

const GC_MARKED: usize = 0;
const GC_SHUNTED: usize = 1 << (std::mem::size_of::<usize>() * 8 - 1);
const GC_UNMARKED: usize = usize::MAX >> 1;

pub struct GarbageCollector {
    map: Vec<usize>,
    trail_map: Vec<usize>,
    scheduler: GCScheduler,
}

pub struct GCScheduler {
    absolute_threshold: usize,
    relative_threshold: f64,
    cooldown: usize,
    remaining_cooldown: usize,
}

pub trait GCRewritable {
    fn rewrite(&mut self, map: &[usize], trail_map: &[usize]);
}

impl GarbageCollector {
    pub fn new(absolute_threshold: usize, relative_threshold: f64, cooldown: usize) -> Self {
        Self {
            map: Vec::new(),
            trail_map: Vec::new(),
            scheduler: GCScheduler {
                absolute_threshold,
                relative_threshold,
                cooldown,
                remaining_cooldown: 0,
            },
        }
    }

    pub fn disabled() -> Self {
        Self {
            map: Vec::new(),
            trail_map: Vec::new(),
            scheduler: GCScheduler {
                absolute_threshold: usize::MAX,
                relative_threshold: 0.0,
                cooldown: 0,
                remaining_cooldown: 0,
            },
        }
    }

    pub fn run(solver: &mut Solver) {
        let roots = solver
            .gc
            .get_roots(&solver.var_map, &solver.goals, &solver.choice_points);
        solver
            .gc
            .collect(&mut solver.heap, &mut solver.trail, roots);
        solver.gc.rewrite(
            &mut solver.var_map,
            &mut solver.goals,
            &mut solver.choice_points,
        );
    }

    pub fn get_roots<'a>(
        &self,
        vars: &'a [(String, usize)],
        goals: &'a Goals,
        choice_points: &'a [ChoicePoint],
    ) -> impl Iterator<Item = HeapTermPtr> + 'a {
        vars.iter()
            .map(|(_, ptr)| *ptr)
            .chain(goals.iter())
            .chain(choice_points.iter().flat_map(|cp| {
                goals
                    .iter_from(cp.goals_checkpoint)
                    .chain(std::iter::once(cp.heap_checkpoint.0))
            }))
    }

    pub fn collect(
        &mut self,
        heap: &mut Heap,
        trail: &mut Trail,
        roots: impl Iterator<Item = HeapTermPtr>,
    ) {
        self.map.clear();
        self.map.resize(heap.data.len(), GC_UNMARKED);

        for root in roots {
            self.mark(heap, root);
        }

        self.shunt(heap);
        self.compact(heap, trail);
    }

    fn mark(&mut self, heap: &Heap, ptr: HeapTermPtr) {
        if self.map[ptr] == GC_MARKED {
            return;
        }

        self.map[ptr] = GC_MARKED;

        match &heap.data[ptr] {
            HeapTerm::Var(ptr, _) => self.mark(heap, *ptr),
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

    fn shunt(&mut self, heap: &Heap) {
        for (i, term) in heap.data.iter().enumerate().rev() {
            if let HeapTerm::Var(ptr, true) = term {
                match self.map[*ptr] {
                    GC_MARKED => self.map[i] = GC_SHUNTED | *ptr, // End of shunted chain
                    GC_UNMARKED => (),                            // Dead variable
                    next => self.map[i] = next,                   // Earlier in shunted chain
                }
            }
        }
    }

    fn compact(&mut self, heap: &mut Heap, trail: &mut Trail) {
        let mut new_ptr: HeapTermPtr = 0;

        // Shuffle data down the heap, overwriting dead data
        for (old_ptr, ptr) in self
            .map
            .iter_mut()
            .enumerate()
            .filter(|(_, ptr)| **ptr == GC_MARKED)
        {
            *ptr = new_ptr;
            new_ptr += 1;

            if *ptr != old_ptr {
                heap.data[*ptr] = heap.data[old_ptr];
            }
        }

        heap.data.truncate(new_ptr);

        // Do the same on the trail stack
        self.collect_trail(trail);

        // Rewrite shunted pointers
        for i in 0..self.map.len() {
            if self.map[i] & GC_SHUNTED != 0 {
                let old_ptr = self.map[i] ^ GC_SHUNTED;
                self.map[i] = self.map[old_ptr];
            }
        }

        // Rewrite internal pointers
        for term in heap.data.iter_mut() {
            match term {
                HeapTerm::Var(ptr, _) => *ptr = self.map[*ptr],
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

    fn collect_trail(&mut self, trail: &mut Trail) {
        self.trail_map.clear();
        self.trail_map.resize(trail.vars.len() + 1, GC_UNMARKED);

        let mut new_ptr = 0;
        let mut old_ptr = 0;

        while old_ptr < trail.vars.len() {
            let ptr = trail.vars[old_ptr];

            self.trail_map[old_ptr] = new_ptr;

            if self.map[ptr] < GC_UNMARKED {
                trail.vars[new_ptr] = self.map[ptr];
                new_ptr += 1;
            }

            old_ptr += 1;
        }

        self.trail_map[old_ptr] = new_ptr;

        trail.vars.truncate(new_ptr);
    }

    pub fn rewrite(
        &self,
        vars: &mut [(String, usize)],
        goals: &mut Goals,
        choice_points: &mut [ChoicePoint],
    ) {
        vars.rewrite(&self.map, &self.trail_map);
        goals.rewrite(&self.map, &self.trail_map);
        choice_points.rewrite(&self.map, &self.trail_map);
    }

    pub fn should_run(&mut self, heap: &Heap) -> bool {
        self.scheduler.should_run(heap)
    }
}

impl GCRewritable for [(String, usize)] {
    fn rewrite(&mut self, map: &[usize], _: &[usize]) {
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

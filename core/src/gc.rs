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
    map_len: usize,
    scheduler: GCScheduler,
}

pub struct GCScheduler {
    absolute_threshold: usize,
    relative_threshold: f64,
    cooldown: usize,
    remaining_cooldown: usize,
    last_live_percentage: f64,
}

pub trait GCRewritable {
    fn rewrite(&mut self, map: &[usize], trail_map: &[usize]);
}

impl GarbageCollector {
    pub fn new(absolute_threshold: usize, relative_threshold: f64, cooldown: usize) -> Self {
        Self {
            map: Vec::new(),
            trail_map: Vec::new(),
            map_len: 0,
            scheduler: GCScheduler {
                absolute_threshold,
                relative_threshold,
                cooldown,
                remaining_cooldown: 0,
                last_live_percentage: 0.0,
            },
        }
    }

    pub fn disabled() -> Self {
        Self {
            map: Vec::new(),
            trail_map: Vec::new(),
            map_len: 0,
            scheduler: GCScheduler {
                absolute_threshold: usize::MAX,
                relative_threshold: 0.0,
                cooldown: 0,
                remaining_cooldown: 0,
                last_live_percentage: 0.0,
            },
        }
    }

    pub fn run(solver: &mut Solver) {
        solver
            .gc
            .reset(solver.heap.data.len(), solver.trail.vars.len());

        let roots = solver.gc.get_roots(&solver.var_map, &solver.goals);

        solver.gc.mark_heap(&solver.heap, roots);
        solver.gc.mark_from_choice_points(
            &mut solver.heap,
            &solver.goals,
            &mut solver.trail,
            &solver.choice_points,
        );

        solver.gc.shunt(&solver.heap);
        solver.gc.realign_choice_points(&mut solver.choice_points);
        solver.gc.compact(&mut solver.heap, &mut solver.trail);

        solver.gc.scheduler.last_live_percentage =
            (solver.heap.data.len() as f64) / (solver.gc.map.len() as f64);

        solver.gc.rewrite(
            &mut solver.var_map,
            &mut solver.goals,
            &mut solver.choice_points,
        );
    }

    fn reset(&mut self, heap_len: usize, trail_len: usize) {
        self.map.clear();
        self.map.resize(heap_len + 1, GC_UNMARKED);
        self.map_len = heap_len;

        self.trail_map.clear();
        self.trail_map.resize(trail_len + 1, 0);
    }

    fn get_roots<'a>(
        &self,
        vars: &'a [(String, usize)],
        goals: &'a Goals,
    ) -> impl Iterator<Item = HeapTermPtr> + 'a {
        vars.iter().map(|(_, ptr)| *ptr).chain(goals.iter())
    }

    fn mark_heap(&mut self, heap: &Heap, roots: impl Iterator<Item = HeapTermPtr>) {
        for root in roots {
            self.mark(heap, root);
        }
    }

    fn mark_from_choice_points(
        &mut self,
        heap: &mut Heap,
        goals: &Goals,
        trail: &mut Trail,
        choice_points: &[ChoicePoint],
    ) {
        let mut last_top = trail.vars.len();

        for cp in choice_points.iter().rev() {
            // Early reset variables that will be reset when backtracking to
            // this choice point and are not otherwise reachable
            for var in (cp.trail_checkpoint.0..last_top).map(|i| trail.vars[i]) {
                if self.map[var] == GC_UNMARKED {
                    heap.unbind(var);
                }
            }

            for goal in goals.iter_from(cp.goals_checkpoint) {
                self.mark(heap, goal);
            }

            last_top = cp.trail_checkpoint.0;
        }
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

    fn realign_choice_points(&mut self, choice_points: &mut [ChoicePoint]) {
        for cp in choice_points.iter_mut() {
            let mut checkpoint = cp.heap_checkpoint.0;

            while checkpoint < self.map_len
                && (self.map[checkpoint] == GC_UNMARKED || self.map[checkpoint] & GC_SHUNTED != 0)
            {
                checkpoint += 1;
            }

            cp.heap_checkpoint.0 = checkpoint;
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

        *self.map.last_mut().unwrap() = new_ptr;
        heap.data.truncate(new_ptr);

        // Do the same on the trail stack
        self.collect_trail(trail);

        // Rewrite shunted pointers
        for i in 0..self.map_len {
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
            self.remaining_cooldown =
                (self.cooldown as f64 * (1.0 / (1.0 - self.last_live_percentage))) as usize;
        } else {
            self.remaining_cooldown = self.remaining_cooldown.saturating_sub(1);
        }

        should_run
    }
}

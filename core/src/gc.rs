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
    runs: usize,

    // Generational GC
    start_choice_point: usize,
    start_heap_ptr: crate::heap::Checkpoint,
    start_trail_ptr: crate::trail::Checkpoint,
    start_goal_ptr: crate::goal::Checkpoint,
}

pub struct GCScheduler {
    absolute_threshold: usize,
    relative_threshold: f64,
    wait_for_resize: Option<usize>,
    cooldown: usize,
    remaining_cooldown: usize,
}

pub trait GCRewritable {
    fn rewrite(&mut self, from: usize, map: &[usize], trail_map: &[usize]);
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
                wait_for_resize: None,
                cooldown,
                remaining_cooldown: 0,
            },
            runs: 0,
            start_choice_point: 0,
            start_heap_ptr: crate::heap::Checkpoint(0),
            start_trail_ptr: crate::trail::Checkpoint(0),
            start_goal_ptr: crate::goal::Checkpoint(None, 0),
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
                wait_for_resize: None,
                cooldown: 0,
                remaining_cooldown: 0,
            },
            runs: 0,
            start_choice_point: 0,
            start_heap_ptr: crate::heap::Checkpoint(0),
            start_trail_ptr: crate::trail::Checkpoint(0),
            start_goal_ptr: crate::goal::Checkpoint(None, 0),
        }
    }

    pub fn runs(&self) -> usize {
        self.runs
    }

    pub fn run(solver: &mut Solver) {
        solver
            .gc
            .reset(solver.heap.data.len(), solver.trail.vars.len());

        if solver.gc.start_choice_point > 0 {
            let cp = &solver.choice_points[solver.gc.start_choice_point - 1];
            solver.gc.start_heap_ptr = cp.heap_checkpoint;
            solver.gc.start_trail_ptr = cp.trail_checkpoint;
            solver.gc.start_goal_ptr = cp.goals_checkpoint;
        } else {
            solver.gc.start_heap_ptr = crate::heap::Checkpoint(0);
            solver.gc.start_trail_ptr = crate::trail::Checkpoint(0);
            solver.gc.start_goal_ptr = crate::goal::Checkpoint(None, 0);
        }

        solver.gc.start_heap_ptr =
            crate::heap::Checkpoint(solver.gc.start_heap_ptr.0.max(solver.heap.code_end));

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
        solver.gc.reset_old_generation_maps();

        solver.gc.compact(&mut solver.heap, &mut solver.trail);

        solver.gc.rewrite(
            &mut solver.var_map,
            &mut solver.goals,
            &mut solver.choice_points,
        );

        solver
            .gc
            .update_old_pointers(&mut solver.heap, &solver.trail);

        solver.gc.scheduler.post_run(&solver.heap);
        solver.gc.runs += 1;
        solver.gc.start_choice_point = solver.choice_points.len();
    }

    fn reset(&mut self, heap_len: usize, trail_len: usize) {
        self.map.clear();
        self.map.resize(heap_len + 1, GC_UNMARKED);
        self.map_len = heap_len;

        self.trail_map.clear();
        self.trail_map.resize(trail_len + 1, GC_UNMARKED);
    }

    fn get_roots<'a>(
        &self,
        vars: &'a [(String, HeapTermPtr)],
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

        for cp in choice_points.iter().skip(self.start_choice_point).rev() {
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

    fn mark(&mut self, heap: &Heap, mut ptr: HeapTermPtr) {
        loop {
            if self.map[ptr] == GC_MARKED {
                return;
            }

            self.map[ptr] = GC_MARKED;

            match &heap.data[ptr] {
                HeapTerm::Var(next, _) => ptr = *next,
                HeapTerm::Compound(_, arity) if *arity > 0 => {
                    for i in 1..=(arity - 1) {
                        self.mark(heap, ptr + i);
                    }
                    ptr += *arity; // tail recursion to avoid stack overflow on lists
                }
                _ => return,
            }
        }
    }

    fn shunt(&mut self, heap: &Heap) {
        for (i, term) in heap
            .data
            .iter()
            .enumerate()
            .skip(self.start_heap_ptr.0)
            .rev()
        {
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
        for cp in choice_points.iter_mut().skip(self.start_choice_point) {
            let mut checkpoint = cp.heap_checkpoint.0;

            while checkpoint < self.map_len
                && (self.map[checkpoint] == GC_UNMARKED || self.map[checkpoint] & GC_SHUNTED != 0)
            {
                checkpoint += 1;
            }

            cp.heap_checkpoint.0 = checkpoint;
        }
    }

    fn reset_old_generation_maps(&mut self) {
        // Old generations haven't changed
        for i in 0..self.start_heap_ptr.0 {
            self.map[i] = i;
        }
        for i in 0..self.start_trail_ptr.0 {
            self.trail_map[i] = i;
        }
    }

    fn compact(&mut self, heap: &mut Heap, trail: &mut Trail) {
        let mut new_ptr: HeapTermPtr = self.start_heap_ptr.0;

        // Shuffle data down the heap, overwriting dead data
        for (old_ptr, ptr) in self
            .map
            .iter_mut()
            .enumerate()
            .skip(self.start_heap_ptr.0)
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
        for i in self.start_heap_ptr.0..self.map_len {
            if self.map[i] & GC_SHUNTED != 0 {
                let old_ptr = self.map[i] ^ GC_SHUNTED;
                self.map[i] = self.map[old_ptr];
            }
        }

        // Rewrite internal pointers
        for term in heap.data.iter_mut().skip(self.start_heap_ptr.0) {
            if let HeapTerm::Var(ptr, _) = term {
                *ptr = self.map[*ptr]
            }
        }
    }

    fn collect_trail(&mut self, trail: &mut Trail) {
        let mut new_ptr = self.start_trail_ptr.0;
        let mut old_ptr = self.start_trail_ptr.0;

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

    fn rewrite(
        &mut self,
        vars: &mut [(String, usize)],
        goals: &mut Goals,
        choice_points: &mut [ChoicePoint],
    ) {
        vars.rewrite(0, &self.map, &self.trail_map);
        goals.rewrite(self.start_goal_ptr.1, &self.map, &self.trail_map);
        choice_points.rewrite(self.start_choice_point, &self.map, &self.trail_map);
    }

    // The trail stack conveniently tells us exactly where to find all the
    // pointers from the old generation to the new generation.
    fn update_old_pointers(&mut self, heap: &mut Heap, trail: &Trail) {
        for var in trail.vars.iter().skip(self.start_trail_ptr.0) {
            if let HeapTerm::Var(ptr, _) = &mut heap.data[*var] {
                if *var < self.start_heap_ptr.0
                    && *ptr >= self.start_heap_ptr.0
                    && self.map[*ptr] < GC_UNMARKED
                {
                    *ptr = self.map[*ptr];
                }
            }
        }
    }

    pub fn pre_run(&mut self, heap: &Heap, choice_points: usize) -> bool {
        self.start_choice_point = self.start_choice_point.min(choice_points);

        self.scheduler.pre_run(heap)
    }
}

impl GCRewritable for [(String, usize)] {
    fn rewrite(&mut self, from: usize, map: &[usize], _: &[usize]) {
        for (_, ptr) in self.iter_mut().skip(from) {
            *ptr = map[*ptr];
        }
    }
}

impl GCScheduler {
    fn pre_run(&mut self, heap: &Heap) -> bool {
        let result = heap.data.len() > self.absolute_threshold
            && self
                .wait_for_resize
                .map(|capacity| capacity != heap.capacity())
                .unwrap_or(true)
            && (heap.size() as f64) / (heap.capacity() as f64) > self.relative_threshold
            && self.remaining_cooldown == 0;

        self.remaining_cooldown = self.remaining_cooldown.saturating_sub(1);

        result
    }

    fn post_run(&mut self, heap: &Heap) {
        self.wait_for_resize = if heap.data.len() > self.absolute_threshold
            && (heap.size() as f64) / (heap.capacity() as f64) > self.relative_threshold
        {
            Some(heap.capacity())
        } else {
            None
        };

        self.remaining_cooldown = self.cooldown;
    }
}

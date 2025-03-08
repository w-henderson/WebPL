use crate::{
    ast, Atom, ClauseName, Heap, HeapClausePtr, HeapTerm, HeapTermPtr, Index, Lambda, LambdaId,
    StringId,
};

pub fn compile(ast_program: ast::Program, heap: &mut Heap, lambdas: &mut Vec<Lambda>) -> Index {
    let mut index: Vec<(ClauseName, Vec<HeapClausePtr>)> = Vec::new();

    let mut var_map = Vec::new();
    for ast_clause in ast_program.0 {
        var_map.clear();

        let goals = heap.data.len();
        let goals_length = ast_clause.1.len();
        for _ in &ast_clause.1 {
            heap.alloc_new_var();
        }

        let (head, clause_name) = ast_clause.0.alloc(heap, &mut var_map, lambdas);
        let head_length = heap.data.len() - head;
        let clause_name = clause_name.unwrap();

        let group_index = index.iter().position(|(name, _)| *name == clause_name);
        let group = if let Some(group_index) = group_index {
            &mut index[group_index].1
        } else {
            index.push((clause_name, Vec::new()));
            &mut index.last_mut().unwrap().1
        };

        let body = ast_clause.alloc_body(heap, goals, &mut var_map, lambdas);
        let body_length = heap.data.len() - body;

        group.push(HeapClausePtr {
            ptr: goals,
            head_length,
            goals_length,
            body_length,
        });
    }

    heap.code_end = heap.data.len();

    index
}

pub fn alloc_query(
    ast_query: ast::Query,
    heap: &mut Heap,
    lambdas: &mut Vec<Lambda>,
) -> (Vec<HeapTermPtr>, Vec<(String, HeapTermPtr)>) {
    let mut var_map = Vec::new();
    let mut heap_query = Vec::new();

    for term in &ast_query.0 {
        heap_query.push(term.alloc(heap, &mut var_map, lambdas).0);
    }

    let var_map = var_map
        .into_iter()
        .map(|(id, ptr)| (heap.string_map.get(id).unwrap().to_string(), ptr))
        .collect();

    (heap_query, var_map)
}

impl ast::Term {
    pub fn alloc(
        &self,
        heap: &mut Heap,
        var_map: &mut Vec<(StringId, HeapTermPtr)>,
        lambdas: &mut Vec<Lambda>,
    ) -> (HeapTermPtr, Option<ClauseName>) {
        match self {
            Self::Atom(atom) => {
                let atom = Atom::new(&mut heap.string_map, atom);
                let ptr = heap.alloc(HeapTerm::Atom(atom));
                if let Atom::String(string_id) = &atom {
                    (ptr, Some(ClauseName(*string_id, 0)))
                } else {
                    (ptr, None)
                }
            }
            Self::Variable(var) if var == "_" => (heap.alloc_new_var(), None),
            Self::Variable(var) => {
                let var = heap.string_map.alloc(var);

                if let Some((_, unified)) = var_map.iter().find(|(x, _)| *x == var) {
                    (heap.alloc(HeapTerm::Var(*unified, true, 0)), None) // shunted
                } else {
                    let result = heap.alloc_new_var();
                    var_map.push((var, result));
                    (result, None)
                }
            }
            Self::Compound(functor, args) => {
                let functor = heap.string_map.alloc(functor);
                let arity = args.len();
                let result = heap.alloc(HeapTerm::Compound(functor, arity));

                let args_heap = heap.data.len();
                for _ in args {
                    heap.alloc_new_var();
                }

                for (i, arg) in args.iter().enumerate() {
                    let (arg, _) = arg.alloc(heap, var_map, lambdas);
                    heap.data[args_heap + i] = HeapTerm::Var(arg, false, 0);
                }

                (result, Some(ClauseName(functor, args.len())))
            }
            Self::Lambda(js, args) => {
                let lambda_id: LambdaId = lambdas.len();
                lambdas.push(Lambda {
                    js: js.clone(),
                    arg_names: args.clone(),
                });

                let arity = args.len();
                let result = heap.alloc(HeapTerm::Lambda(lambda_id, arity));

                let args_heap = heap.data.len();
                for _ in args {
                    heap.alloc_new_var();
                }

                for (i, arg) in args.iter().enumerate() {
                    let (arg, _) = ast::Term::Variable(arg.clone()).alloc(heap, var_map, lambdas);
                    heap.data[args_heap + i] = HeapTerm::Var(arg, false, 0);
                }

                (result, None)
            }
            Self::Cut => (heap.alloc(HeapTerm::Cut(0)), None),
        }
    }
}

impl ast::Clause {
    pub fn alloc_body(
        &self,
        heap: &mut Heap,
        mut goals: HeapTermPtr,
        var_map: &mut Vec<(StringId, HeapTermPtr)>,
        lambdas: &mut Vec<Lambda>,
    ) -> HeapTermPtr {
        let result = heap.data.len();

        for goal in &self.1 {
            let (goal, _) = goal.alloc(heap, var_map, lambdas);
            heap.data[goals] = HeapTerm::Var(goal, false, 0);
            goals += 1;
        }

        result
    }
}

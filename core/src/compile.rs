use crate::stringmap::StringMap;
use crate::{ast, Clause, ClauseName, Program};

pub fn compile(ast_program: ast::Program, string_map: &mut StringMap) -> Program {
    let mut program: Vec<(ClauseName, Vec<Clause>)> = Vec::new();

    for ast_clause in ast_program.0 {
        let (head, clause_name) = ast_clause.0.to_code_term(string_map);
        let clause_name = clause_name.unwrap();

        let group_index = program.iter().position(|(name, _)| *name == clause_name);
        let group = if let Some(index) = group_index {
            &mut program[index].1
        } else {
            program.push((clause_name, Vec::new()));
            &mut program.last_mut().unwrap().1
        };

        group.push(Clause {
            head,
            body: ast_clause
                .1
                .into_iter()
                .map(|term| term.to_code_term(string_map).0)
                .collect(),
        });
    }

    program
}

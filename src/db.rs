#![allow(non_camel_case_types)]

use crate::error::ChalkError;
use crate::lowering::LowerGoal;
use crate::program::Program;
use crate::query::{Lowering, LoweringDatabase};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::tls;
use chalk_ir::DomainGoal;
use chalk_ir::Goal;
use chalk_ir::IsCoinductive;
use chalk_ir::ProgramClause;
use chalk_ir::TraitId;
use chalk_solve::solve::ProgramClauseSet;
use chalk_solve::solve::SolverChoice;
use salsa::Database;
use std::sync::Arc;

#[salsa::database(Lowering)]
#[derive(Debug, Default)]
pub struct ChalkDatabase {
    runtime: salsa::Runtime<ChalkDatabase>,
}

impl Database for ChalkDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<ChalkDatabase> {
        &self.runtime
    }
}

impl ChalkDatabase {
    pub fn with(program_text: &str, solver_choice: SolverChoice) -> Self {
        let mut db = ChalkDatabase::default();
        db.set_program_text(Arc::new(program_text.to_string()));
        db.set_solver_choice(solver_choice);
        db
    }

    pub fn with_program<R>(&self, op: impl FnOnce(&Program) -> R) -> R {
        let program = &self.checked_program().unwrap();
        tls::set_current_program(&program, || op(&program))
    }

    pub fn parse_and_lower_goal(&self, text: &str) -> Result<Box<Goal>, ChalkError> {
        let program = self.checked_program()?;
        Ok(chalk_parse::parse_goal(text)?.lower(&*program)?)
    }
}

impl ProgramClauseSet for ChalkDatabase {
    fn program_clauses_that_could_match(&self, goal: &DomainGoal, vec: &mut Vec<ProgramClause>) {
        if let Ok(env) = self.environment() {
            vec.extend(
                env.program_clauses
                    .iter()
                    .filter(|&clause| clause.could_match(goal))
                    .cloned(),
            );
        }
    }

    fn upcast(&self) -> &dyn IsCoinductive {
        self
    }
}

impl IsCoinductive for ChalkDatabase {
    fn is_coinductive_trait(&self, trait_id: TraitId) -> bool {
        if let Ok(env) = self.environment() {
            env.coinductive_traits.contains(&trait_id)
        } else {
            false
        }
    }
}

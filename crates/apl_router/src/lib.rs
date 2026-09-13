use apl_core::Program;

#[derive(Debug, Clone)]
pub struct RouterPlan {
    pub program: Program,
}

impl RouterPlan {
    pub fn from_checked_program(program: Program) -> Self {
        Self { program }
    }
}

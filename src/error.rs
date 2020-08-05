use derive_more::Display;

// TODO: unify error handling around this module

/// Errors produced by Techalyzer main program
#[derive(Debug, Display)]
pub enum TechalyzerError {
    #[display(fmt = "{}", _0)]
    Generic(String),
}

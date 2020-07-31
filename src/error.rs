use derive_more::Display;

/// Errors produced by Techalyzer main program
#[derive(Debug, Display)]
pub enum TechalyzerError {
    #[display(fmt = "{}", _0)]
    Generic(String),
}
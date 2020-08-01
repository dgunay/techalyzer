// TODO: this should probably be shared with SupportedDataSources
/// The set of APIs or other finance data sources.
pub enum Source {
    TechalyzerJson(std::path::PathBuf),
    AlphaVantage,
}

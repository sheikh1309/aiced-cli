#[derive(Debug, Clone)]
pub enum AnalysisStatus {
    Success,
    PartialSuccess(String),
    Failed(()),
}
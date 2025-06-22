#[derive(Debug, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn print_summary(&self) {
        if self.is_valid {
            log::info!("✅ Validation passed");
        } else {
            log::info!("❌ Validation failed with {} errors", self.errors.len());
        }

        if !self.warnings.is_empty() {
            log::info!("⚠️ {} warnings found", self.warnings.len());
        }

        for error in &self.errors {
            log::info!("   ❌ {}", error);
        }

        for warning in &self.warnings {
            log::info!("   ⚠️ {}", warning);
        }
    }
}
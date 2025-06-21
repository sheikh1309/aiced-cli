#[derive(Debug, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn print_summary(&self) {
        if self.is_valid {
            println!("✅ Validation passed");
        } else {
            println!("❌ Validation failed with {} errors", self.errors.len());
        }

        if !self.warnings.is_empty() {
            println!("⚠️ {} warnings found", self.warnings.len());
        }

        for error in &self.errors {
            println!("   ❌ {}", error);
        }

        for warning in &self.warnings {
            println!("   ⚠️ {}", warning);
        }
    }
}
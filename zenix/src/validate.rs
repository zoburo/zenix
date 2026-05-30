//! Validation integration powered by [`garde`](https://docs.rs/garde).
//!
//! Re-exports the [`garde::Validate`] derive macro and provides helpers to
//! convert validation errors into HTTP-friendly JSON responses.
//!
//! # Example
//!
//! ```ignore
//! use garde::Validate;
//! use zenix::validate::ReportExt;
//!
//! #[derive(Validate)]
//! struct CreateUser {
//!     #[garde(required, length(min = 3, max = 100))]
//!     name: String,
//!
//!     #[garde(required, email)]
//!     email: String,
//!
//!     #[garde(range(min = 18, max = 150))]
//!     age: u8,
//! }
//!
//! let input = CreateUser { ... };
//! if let Err(report) = input.validate() {
//!     let body = report.to_json();
//! }
//! ```

// ── Re-exports ───────────────────────────────────────────────────────────────

/// Re-export of [`garde::Validate`] — the core validation trait + derive macro.
pub use garde::Validate;

/// Re-export of [`garde::Report`] — a collection of field-level validation errors.
pub use garde::Report;

/// Re-export of [`garde::Error`] — a single validation error with a message.
pub use garde::Error;

/// Re-export of [`garde::Path`] — the path to a validated field (e.g. `"name"`, `"address.city"`).
pub use garde::Path;

// ── JSON conversion ──────────────────────────────────────────────────────────

/// Extension trait adding [`to_json`](ReportExt::to_json) to [`garde::Report`].
pub trait ReportExt {
    /// Render the validation report as a JSON string suitable for API responses.
    ///
    /// The output groups error messages by field:
    ///
    /// ```json
    /// {
    ///   "name": ["name is required"],
    ///   "email": ["email is not a valid email address"],
    ///   "age": ["age must be between 18 and 150 (got 12)"]
    /// }
    /// ```
    fn to_json(&self) -> String;
}

impl ReportExt for garde::Report {
    fn to_json(&self) -> String {
        let mut map = serde_json::Map::new();
        for (path, error) in self.iter() {
            let field = path.to_string();
            let entry = map
                .entry(&field)
                .or_insert_with(|| serde_json::Value::Array(vec![]));
            if let serde_json::Value::Array(arr) = entry {
                arr.push(serde_json::Value::String(error.message().to_string()));
            }
        }
        serde_json::Value::Object(map).to_string()
    }
}

/// Convenience function to convert a [`garde::Report`] into a JSON string.
///
/// Equivalent to calling `report.to_json()` via the [`ReportExt`] trait.
pub fn errors_to_json(report: &garde::Report) -> String {
    report.to_json()
}

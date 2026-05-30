// ── Modules ──────────────────────────────────────────────────────────────────

mod route;

pub mod context;
pub mod extract;
pub mod handler;
pub mod server;
pub mod validate;

// ── Public re-exports ────────────────────────────────────────────────────────

pub use context::{Context, StatusResponder};
pub use handler::Response;
pub use server::Server;
pub use validate::{ReportExt, Validate};

// Re-export the derive macro.  The trait lives at [`extract::FromRequest`].
pub use zenix_derive::FromRequest;

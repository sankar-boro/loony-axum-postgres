pub mod doc;

/// User ID injected into request extensions by the `require_auth` middleware.
#[derive(Clone, Copy)]
pub struct UserId(pub i32);

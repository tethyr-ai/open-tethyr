//! Authentication Provider Templates
//!
//! This module provides authentication provider templates, currently focused on OAuth
//! providers for generating provider-specific endpoint configurations.
//!
//! MVP includes: Okta, Auth0, and Generic OAuth2 providers.
//! Future versions will expand to include mTLS, JWT, and additional OAuth providers.

mod auth0;
mod generic;
mod okta;
mod provider;
mod registry;
mod types;

// OAuth providers (MVP implementation)
pub use auth0::Auth0Provider;
pub use generic::GenericOAuth2Provider;
pub use okta::OktaProvider;
pub use provider::OAuthProvider;
pub use registry::ProviderRegistry;
pub use types::{OAuthConfig, OAuthEndpoints};

// Future: mTLS, JWT, API Key, additional OAuth providers
// pub mod mtls;
// pub mod jwt;
// pub mod api_key;
// pub mod azure;     // Enterprise expansion
// pub mod cognito;   // AWS expansion
// pub mod google;    // Google Workspace expansion
// pub mod keycloak;  // Self-hosted expansion

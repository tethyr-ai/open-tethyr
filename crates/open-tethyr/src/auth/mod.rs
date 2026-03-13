//! OAuth Provider Templates

pub mod auth0;
pub mod generic;
pub mod okta;
pub mod provider;

pub use auth0::Auth0Provider;
pub use generic::GenericOAuth2Provider;
pub use okta::OktaProvider;
pub use provider::{OAuthProvider, ProviderRegistry};

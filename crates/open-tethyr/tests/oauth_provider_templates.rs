//! Property tests for OAuth provider templates
//!
//! **Property 9: OAuth Provider Template Correctness**
//! **Validates: Requirements 6.1, 6.2, 6.7**

use open_tethyr::auth::{
    Auth0Provider, GenericOAuth2Provider, OAuthConfig, OAuthProvider, OktaProvider,
    ProviderRegistry,
};
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

// Property test generators for valid OAuth configurations
fn arb_valid_okta_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("dev-123.okta.com".to_string()),
        Just("acme.okta.com".to_string()),
        Just("test-org.okta.com".to_string()),
        Just("company.oktapreview.com".to_string()),
        Just("my-org.okta-emea.com".to_string()),
    ]
}

fn arb_valid_auth0_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("acme.auth0.com".to_string()),
        Just("company.us.auth0.com".to_string()),
        Just("test-tenant.eu.auth0.com".to_string()),
        Just("my-app.auth0.com".to_string()),
        Just("dev-env.au.auth0.com".to_string()),
    ]
}

fn arb_valid_generic_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("auth.example.com".to_string()),
        Just("sso.company.com".to_string()),
        Just("identity.acme.org".to_string()),
        Just("oauth.service.net".to_string()),
        Just("login.myapp.io".to_string()),
    ]
}

fn arb_invalid_okta_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("example.com".to_string()),          // Missing okta.com
        Just("auth0.com".to_string()),            // Wrong provider
        Just("https://dev.okta.com".to_string()), // Has protocol
        Just("".to_string()),                     // Empty
        Just("okta.com".to_string()),             // Just base domain
    ]
}

fn arb_invalid_auth0_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("example.com".to_string()),            // Missing auth0.com
        Just("okta.com".to_string()),               // Wrong provider
        Just("https://acme.auth0.com".to_string()), // Has protocol
        Just("".to_string()),                       // Empty
        Just("auth0.com".to_string()),              // Just base domain
    ]
}

fn arb_valid_oauth_config_okta() -> impl Strategy<Value = OAuthConfig> {
    (
        arb_valid_okta_domain(),
        prop::option::of("[a-zA-Z0-9]{10,32}"),
    )
        .prop_map(|(domain, client_id)| OAuthConfig {
            domain: Some(domain),
            client_id,
            additional_config: HashMap::new(),
        })
}

fn arb_valid_oauth_config_auth0() -> impl Strategy<Value = OAuthConfig> {
    (
        arb_valid_auth0_domain(),
        prop::option::of("[a-zA-Z0-9]{10,32}"),
    )
        .prop_map(|(domain, client_id)| OAuthConfig {
            domain: Some(domain),
            client_id,
            additional_config: HashMap::new(),
        })
}

fn arb_valid_oauth_config_generic() -> impl Strategy<Value = OAuthConfig> {
    arb_valid_generic_domain().prop_map(|domain| OAuthConfig {
        domain: Some(domain),
        client_id: Some("test-client-id".to_string()),
        additional_config: HashMap::new(),
    })
}

fn arb_valid_oauth_config_generic_manual() -> impl Strategy<Value = OAuthConfig> {
    arb_valid_generic_domain().prop_map(|domain| {
        let mut additional_config = HashMap::new();
        let base_url = format!("https://{}", domain);
        additional_config.insert("issuer".to_string(), base_url.clone());
        additional_config.insert(
            "authorization_endpoint".to_string(),
            format!("{}/auth", base_url),
        );
        additional_config.insert("token_endpoint".to_string(), format!("{}/token", base_url));
        additional_config.insert("jwks_uri".to_string(), format!("{}/jwks", base_url));
        additional_config.insert(
            "userinfo_endpoint".to_string(),
            format!("{}/userinfo", base_url),
        );

        OAuthConfig {
            domain: Some(domain),
            client_id: Some("test-client-id".to_string()),
            additional_config,
        }
    })
}

proptest! {
    /// Property 9: OAuth Provider Template Correctness - Okta Provider
    /// For any valid Okta configuration, endpoint generation should succeed and produce valid Okta endpoints
    /// **Validates: Requirements 6.1, 6.7**
    #[test]
    fn test_okta_provider_generates_valid_endpoints(
        config in arb_valid_oauth_config_okta()
    ) {
        let provider = OktaProvider::new();
        let result = provider.generate_endpoints(&config);

        prop_assert!(result.is_ok(), "Valid Okta config should generate endpoints: {:?}", result);

        let endpoints = result.unwrap();
        let domain = config.domain.as_ref().unwrap();
        let expected_base = format!("https://{}", domain);

        // Verify Okta-specific endpoint patterns
        prop_assert_eq!(endpoints.issuer, expected_base.clone());
        prop_assert_eq!(endpoints.authorization_endpoint, format!("{}/oauth2/authorize", expected_base));
        prop_assert_eq!(endpoints.token_endpoint, format!("{}/oauth2/token", expected_base));
        prop_assert_eq!(endpoints.jwks_uri, format!("{}/oauth2/v1/keys", expected_base));
        prop_assert_eq!(endpoints.userinfo_endpoint, Some(format!("{}/oauth2/v1/userinfo", expected_base)));
        prop_assert_eq!(endpoints.revocation_endpoint, Some(format!("{}/oauth2/v1/revoke", expected_base)));
        prop_assert_eq!(provider.provider_name(), "okta");
    }

    /// Property 9: OAuth Provider Template Correctness - Auth0 Provider
    /// For any valid Auth0 configuration, endpoint generation should succeed and produce valid Auth0 endpoints
    /// **Validates: Requirements 6.2, 6.7**
    #[test]
    fn test_auth0_provider_generates_valid_endpoints(
        config in arb_valid_oauth_config_auth0()
    ) {
        let provider = Auth0Provider::new();
        let result = provider.generate_endpoints(&config);

        prop_assert!(result.is_ok(), "Valid Auth0 config should generate endpoints: {:?}", result);

        let endpoints = result.unwrap();
        let domain = config.domain.as_ref().unwrap();
        let expected_base = format!("https://{}", domain);

        // Verify Auth0-specific endpoint patterns
        prop_assert_eq!(endpoints.issuer, format!("{}/", expected_base)); // Auth0 includes trailing slash
        prop_assert_eq!(endpoints.authorization_endpoint, format!("{}/authorize", expected_base));
        prop_assert_eq!(endpoints.token_endpoint, format!("{}/oauth/token", expected_base));
        prop_assert_eq!(endpoints.jwks_uri, format!("{}/.well-known/jwks.json", expected_base));
        prop_assert_eq!(endpoints.userinfo_endpoint, Some(format!("{}/userinfo", expected_base)));
        prop_assert_eq!(endpoints.revocation_endpoint, Some(format!("{}/oauth/revoke", expected_base)));
        prop_assert_eq!(provider.provider_name(), "auth0");
    }

    /// Property 9: OAuth Provider Template Correctness - Generic Provider (Domain-based)
    /// For any valid generic configuration, endpoint generation should succeed and produce standard OAuth2 endpoints
    /// **Validates: Requirements 6.7**
    #[test]
    fn test_generic_provider_generates_valid_endpoints_from_domain(
        config in arb_valid_oauth_config_generic()
    ) {
        let provider = GenericOAuth2Provider::new();
        let result = provider.generate_endpoints(&config);

        prop_assert!(result.is_ok(), "Valid generic config should generate endpoints: {:?}", result);

        let endpoints = result.unwrap();
        let domain = config.domain.as_ref().unwrap();
        let expected_base = format!("https://{}", domain);

        // Verify standard OAuth2 endpoint patterns
        prop_assert_eq!(endpoints.issuer, expected_base.clone());
        prop_assert_eq!(endpoints.authorization_endpoint, format!("{}/oauth2/authorize", expected_base));
        prop_assert_eq!(endpoints.token_endpoint, format!("{}/oauth2/token", expected_base));
        prop_assert_eq!(endpoints.jwks_uri, format!("{}/.well-known/jwks.json", expected_base));
        prop_assert_eq!(endpoints.userinfo_endpoint, Some(format!("{}/oauth2/userinfo", expected_base)));
        prop_assert_eq!(endpoints.revocation_endpoint, Some(format!("{}/oauth2/revoke", expected_base)));
        prop_assert_eq!(provider.provider_name(), "generic_oauth2");
    }

    /// Property 9: OAuth Provider Template Correctness - Generic Provider (Manual Config)
    /// For any valid manual configuration, endpoint generation should use the provided endpoints
    /// **Validates: Requirements 6.7**
    #[test]
    fn test_generic_provider_generates_valid_endpoints_from_manual_config(
        config in arb_valid_oauth_config_generic_manual()
    ) {
        let provider = GenericOAuth2Provider::new();
        let result = provider.generate_endpoints(&config);

        prop_assert!(result.is_ok(), "Valid manual config should generate endpoints: {:?}", result);

        let endpoints = result.unwrap();

        // Verify manual endpoints are used
        prop_assert_eq!(endpoints.issuer, config.additional_config["issuer"].clone());
        prop_assert_eq!(endpoints.authorization_endpoint, config.additional_config["authorization_endpoint"].clone());
        prop_assert_eq!(endpoints.token_endpoint, config.additional_config["token_endpoint"].clone());
        prop_assert_eq!(endpoints.jwks_uri, config.additional_config["jwks_uri"].clone());
        prop_assert_eq!(endpoints.userinfo_endpoint, config.additional_config.get("userinfo_endpoint").cloned());
    }

    /// Property: Invalid Okta domains should fail validation
    /// **Validates: Requirements 6.1**
    #[test]
    fn test_okta_provider_rejects_invalid_domains(
        invalid_domain in arb_invalid_okta_domain()
    ) {
        let provider = OktaProvider::new();
        let config = OAuthConfig {
            domain: Some(invalid_domain.clone()),
            client_id: None,
            additional_config: HashMap::new(),
        };

        let result = provider.generate_endpoints(&config);
        prop_assert!(result.is_err(), "Invalid Okta domain '{}' should be rejected", invalid_domain);
    }

    /// Property: Invalid Auth0 domains should fail validation
    /// **Validates: Requirements 6.2**
    #[test]
    fn test_auth0_provider_rejects_invalid_domains(
        invalid_domain in arb_invalid_auth0_domain()
    ) {
        let provider = Auth0Provider::new();
        let config = OAuthConfig {
            domain: Some(invalid_domain.clone()),
            client_id: None,
            additional_config: HashMap::new(),
        };

        let result = provider.generate_endpoints(&config);
        prop_assert!(result.is_err(), "Invalid Auth0 domain '{}' should be rejected", invalid_domain);
    }

    /// Property: Provider registry should correctly route to registered providers
    /// **Validates: Requirements 6.7**
    #[test]
    fn test_provider_registry_routes_correctly(
        okta_config in arb_valid_oauth_config_okta(),
        auth0_config in arb_valid_oauth_config_auth0(),
        generic_config in arb_valid_oauth_config_generic()
    ) {
        let mut registry = ProviderRegistry::new();
        registry.register_provider("okta".to_string(), Arc::new(OktaProvider::new()));
        registry.register_provider("auth0".to_string(), Arc::new(Auth0Provider::new()));
        registry.register_provider("generic".to_string(), Arc::new(GenericOAuth2Provider::new()));

        // Test Okta routing
        let okta_result = registry.generate_oauth_config("okta", &okta_config);
        prop_assert!(okta_result.is_ok(), "Registry should route to Okta provider: {:?}", okta_result);

        // Test Auth0 routing
        let auth0_result = registry.generate_oauth_config("auth0", &auth0_config);
        prop_assert!(auth0_result.is_ok(), "Registry should route to Auth0 provider: {:?}", auth0_result);

        // Test Generic routing
        let generic_result = registry.generate_oauth_config("generic", &generic_config);
        prop_assert!(generic_result.is_ok(), "Registry should route to Generic provider: {:?}", generic_result);

        // Test unknown provider
        let unknown_result = registry.generate_oauth_config("unknown", &okta_config);
        prop_assert!(unknown_result.is_err(), "Registry should reject unknown provider");
    }

    /// Property: Missing domain configuration should fail for all providers
    /// **Validates: Requirements 6.1, 6.2, 6.7**
    #[test]
    fn test_all_providers_require_domain(
        client_id in prop::option::of("[a-zA-Z0-9]{10,32}")
    ) {
        let config = OAuthConfig {
            domain: None, // Missing domain
            client_id,
            additional_config: HashMap::new(),
        };

        // Test all providers reject missing domain
        let okta_result = OktaProvider::new().generate_endpoints(&config);
        prop_assert!(okta_result.is_err(), "Okta provider should reject missing domain");

        let auth0_result = Auth0Provider::new().generate_endpoints(&config);
        prop_assert!(auth0_result.is_err(), "Auth0 provider should reject missing domain");

        let generic_result = GenericOAuth2Provider::new().generate_endpoints(&config);
        prop_assert!(generic_result.is_err(), "Generic provider should reject missing domain");
    }

    /// Property: Generated endpoints should always be valid HTTPS URLs
    /// **Validates: Requirements 6.1, 6.2, 6.7**
    #[test]
    fn test_generated_endpoints_are_valid_https_urls(
        okta_config in arb_valid_oauth_config_okta()
    ) {
        let provider = OktaProvider::new();
        let result = provider.generate_endpoints(&okta_config);

        prop_assert!(result.is_ok(), "Valid config should generate endpoints");
        let endpoints = result.unwrap();

        // All endpoints should be valid HTTPS URLs
        prop_assert!(endpoints.issuer.starts_with("https://"), "Issuer should be HTTPS: {}", endpoints.issuer);
        prop_assert!(endpoints.authorization_endpoint.starts_with("https://"), "Authorization endpoint should be HTTPS: {}", endpoints.authorization_endpoint);
        prop_assert!(endpoints.token_endpoint.starts_with("https://"), "Token endpoint should be HTTPS: {}", endpoints.token_endpoint);
        prop_assert!(endpoints.jwks_uri.starts_with("https://"), "JWKS URI should be HTTPS: {}", endpoints.jwks_uri);

        if let Some(ref userinfo) = endpoints.userinfo_endpoint {
            prop_assert!(userinfo.starts_with("https://"), "UserInfo endpoint should be HTTPS: {}", userinfo);
        }

        if let Some(ref revocation) = endpoints.revocation_endpoint {
            prop_assert!(revocation.starts_with("https://"), "Revocation endpoint should be HTTPS: {}", revocation);
        }
    }
}

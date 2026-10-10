//! What a delegated token may reach.
//!
//! A token carrying `act` was obtained by the MCP gateway on behalf of a
//! user. It only ever needs the routes the MCP server calls, so it gets
//! those and nothing else: no member, role or file route, and no way to
//! change an organization's settings, including the switch that lets MCP in.
//! The list is the boundary even if the gateway or the server misbehaves.

use http::Method;

const DELEGATED_ROUTES: &[(Method, &str)] = &[
    (Method::GET, "/api/v1/users/@me/organizations"),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/projects",
    ),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/projects/{project_id}",
    ),
    (
        Method::POST,
        "/api/v1/organizations/{organization_id}/projects/{project_id}/doe",
    ),
    (Method::GET, "/api/v1/organizations/{organization_id}/does"),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/does/{doe_id}",
    ),
    (
        Method::PUT,
        "/api/v1/organizations/{organization_id}/does/{doe_id}",
    ),
    (
        Method::POST,
        "/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft",
    ),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/does/{doe_id}/pdf",
    ),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/doe-models",
    ),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/products",
    ),
    (Method::GET, "/api/v1/products/{product_id}"),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/quotes",
    ),
    (Method::GET, "/api/v1/quotes/{quote_id}"),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/invoices",
    ),
    (Method::GET, "/api/v1/invoices/{invoice_id}"),
    (Method::POST, "/api/v1/files"),
];

/// Whether a delegated token may call `method` on the route template
/// `matched_path` (axum's `MatchedPath`, not the concrete URL).
pub fn delegated_route_allowed(method: &Method, matched_path: &str) -> bool {
    DELEGATED_ROUTES
        .iter()
        .any(|(allowed_method, template)| allowed_method == method && *template == matched_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_routes_the_mcp_server_calls_are_open() {
        assert!(delegated_route_allowed(
            &Method::GET,
            "/api/v1/organizations/{organization_id}/does"
        ));
        assert!(delegated_route_allowed(&Method::POST, "/api/v1/files"));
        assert!(delegated_route_allowed(
            &Method::PUT,
            "/api/v1/organizations/{organization_id}/does/{doe_id}"
        ));
    }

    #[test]
    fn everything_else_is_closed() {
        assert!(!delegated_route_allowed(
            &Method::PATCH,
            "/api/v1/organizations/{organization_id}"
        ));
        assert!(!delegated_route_allowed(
            &Method::DELETE,
            "/api/v1/organizations/{organization_id}/does/{doe_id}"
        ));
        assert!(!delegated_route_allowed(&Method::GET, "/api/v1/files/url"));
        assert!(
            DELEGATED_ROUTES
                .iter()
                .all(|(method, _)| *method != Method::DELETE),
            "no delegated route may delete anything"
        );
    }
}

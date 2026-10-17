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
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/maintenance/recap",
    ),
    (
        Method::GET,
        "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}",
    ),
    (
        Method::POST,
        "/api/v1/organizations/{organization_id}/maintenance/contracts",
    ),
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

    const MAINTENANCE_PREFIX: &str = "/api/v1/organizations/{organization_id}/maintenance";

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

    #[test]
    fn maintenance_opens_the_recap_a_contract_read_and_a_contract_creation_only() {
        let mut open: Vec<(String, &str)> = DELEGATED_ROUTES
            .iter()
            .filter(|(_, template)| template.starts_with(MAINTENANCE_PREFIX))
            .map(|(method, template)| (method.to_string(), *template))
            .collect();
        open.sort();

        let mut expected = vec![
            (
                "GET".to_owned(),
                "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}",
            ),
            (
                "GET".to_owned(),
                "/api/v1/organizations/{organization_id}/maintenance/recap",
            ),
            (
                "POST".to_owned(),
                "/api/v1/organizations/{organization_id}/maintenance/contracts",
            ),
        ];
        expected.sort();
        assert_eq!(open, expected);
    }

    #[test]
    fn the_assistant_can_neither_activate_a_contract_nor_validate_a_visit() {
        let closed = [
            (Method::PUT, "/maintenance/contracts/{contract_id}"),
            (Method::PATCH, "/maintenance/contracts/{contract_id}"),
            (Method::DELETE, "/maintenance/contracts/{contract_id}"),
            (Method::POST, "/maintenance/contracts/{contract_id}/archive"),
            (
                Method::POST,
                "/maintenance/contracts/{contract_id}/duplicate",
            ),
            (Method::GET, "/maintenance/contracts"),
            (Method::GET, "/maintenance/grid"),
            (Method::GET, "/maintenance/recap.pdf"),
            (Method::POST, "/maintenance/recap/plan"),
            (Method::POST, "/maintenance/lines/{line_id}/interventions"),
            (Method::GET, "/maintenance/interventions"),
            (
                Method::POST,
                "/maintenance/interventions/{intervention_id}/validate",
            ),
            (
                Method::POST,
                "/maintenance/interventions/{intervention_id}/reject",
            ),
        ];
        for (method, suffix) in closed {
            let path = format!("{MAINTENANCE_PREFIX}{suffix}");
            assert!(!delegated_route_allowed(&method, &path), "{method} {path}");
        }
    }
}

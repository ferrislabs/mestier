use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use handlers_files as files;
use handlers_organization as organization;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Mestier API", description = "API for Mestier", version = "0.1.0"),
    paths(
        files::upload::handler,
        organization::create::handler,
        organization::list::handler,
        organization::list_mine::handler,
        organization::get_one::handler,
        organization::update::handler,
        organization::soft_delete::handler,
    ),
    components(schemas(
        organization::create::CreateOrganizationRequest,
        organization::update::UpdateOrganizationRequest,
        organization::response::OrganizationResponse,
        files::response::FileUploadResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "organizations", description = "Organizations management"),
        (name = "files", description = "File uploads and storage"),
    )
)]
pub struct ApiDoc;

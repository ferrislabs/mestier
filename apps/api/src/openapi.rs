use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use handlers_files as files;
use handlers_organization as organization;
use handlers_reference as reference;

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
        reference::employee::create::handler,
        reference::employee::list::handler,
        reference::employee::get_one::handler,
        reference::employee::update::handler,
        reference::employee::soft_delete::handler,
        reference::equipment::create::handler,
        reference::equipment::list::handler,
        reference::equipment::get_one::handler,
        reference::equipment::update::handler,
        reference::equipment::soft_delete::handler,
        reference::service_rate::create::handler,
        reference::service_rate::list::handler,
        reference::service_rate::get_one::handler,
        reference::service_rate::update::handler,
        reference::service_rate::soft_delete::handler,
    ),
    components(schemas(
        organization::create::CreateOrganizationRequest,
        organization::update::UpdateOrganizationRequest,
        organization::response::OrganizationResponse,
        files::response::FileUploadResponse,
        reference::employee::create::CreateEmployeeRequest,
        reference::employee::update::UpdateEmployeeRequest,
        reference::equipment::create::CreateEquipmentRequest,
        reference::equipment::update::UpdateEquipmentRequest,
        reference::service_rate::create::CreateServiceRateRequest,
        reference::service_rate::update::UpdateServiceRateRequest,
        reference::response::EmployeeResponse,
        reference::response::EquipmentResponse,
        reference::response::ServiceRateResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "organizations", description = "Organizations management"),
        (name = "files", description = "File uploads and storage"),
        (name = "reference", description = "Reference cost catalog"),
    )
)]
pub struct ApiDoc;

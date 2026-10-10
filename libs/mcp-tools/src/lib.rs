//! The closed catalogue of MCP tools Mestier exposes, and the contract
//! between the MCP gateway and the MCP server. See docs/chantiers/mcp.md.
//!
//! A tool not listed here does not exist: the gateway refuses an unknown
//! name before asking the policy decision point, and the server has nothing
//! to dispatch it to.

use std::{fmt, str::FromStr};

/// OAuth scope granting every read tool.
pub const SCOPE_READ: &str = "mestier.read";
/// OAuth scope granting the DOE write tools.
pub const SCOPE_DOE_WRITE: &str = "mestier.doe.write";

/// Header the gateway sets on requests it forwards, naming the LLM client
/// the call came from. For logs only: authorization never reads it.
pub const CLIENT_HEADER: &str = "x-mestier-mcp-client";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    Read,
    Write,
}

impl Access {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    ListOrganizations,
    ListProjects,
    GetProject,
    ListDoes,
    GetDoe,
    ListDoeModels,
    SearchProducts,
    GetProduct,
    ListQuotes,
    GetQuote,
    ListInvoices,
    GetInvoice,
    CreateDoe,
    AddDoeSupplyFromProduct,
    AddDoeSupply,
    UpdateDoeHeader,
    ExportDoePdf,
}

impl Tool {
    pub const ALL: [Tool; 17] = [
        Tool::ListOrganizations,
        Tool::ListProjects,
        Tool::GetProject,
        Tool::ListDoes,
        Tool::GetDoe,
        Tool::ListDoeModels,
        Tool::SearchProducts,
        Tool::GetProduct,
        Tool::ListQuotes,
        Tool::GetQuote,
        Tool::ListInvoices,
        Tool::GetInvoice,
        Tool::CreateDoe,
        Tool::AddDoeSupplyFromProduct,
        Tool::AddDoeSupply,
        Tool::UpdateDoeHeader,
        Tool::ExportDoePdf,
    ];

    /// The name MCP clients call the tool by.
    pub fn name(self) -> &'static str {
        match self {
            Self::ListOrganizations => "list_organizations",
            Self::ListProjects => "list_projects",
            Self::GetProject => "get_project",
            Self::ListDoes => "list_does",
            Self::GetDoe => "get_doe",
            Self::ListDoeModels => "list_doe_models",
            Self::SearchProducts => "search_products",
            Self::GetProduct => "get_product",
            Self::ListQuotes => "list_quotes",
            Self::GetQuote => "get_quote",
            Self::ListInvoices => "list_invoices",
            Self::GetInvoice => "get_invoice",
            Self::CreateDoe => "create_doe",
            Self::AddDoeSupplyFromProduct => "add_doe_supply_from_product",
            Self::AddDoeSupply => "add_doe_supply",
            Self::UpdateDoeHeader => "update_doe_header",
            Self::ExportDoePdf => "export_doe_pdf",
        }
    }

    pub fn access(self) -> Access {
        match self {
            Self::CreateDoe
            | Self::AddDoeSupplyFromProduct
            | Self::AddDoeSupply
            | Self::UpdateDoeHeader => Access::Write,
            _ => Access::Read,
        }
    }

    /// The business area, as the policy sees it.
    pub fn domain(self) -> &'static str {
        match self {
            Self::ListOrganizations => "organization",
            Self::ListProjects | Self::GetProject => "project",
            Self::ListDoes
            | Self::GetDoe
            | Self::ListDoeModels
            | Self::CreateDoe
            | Self::AddDoeSupplyFromProduct
            | Self::AddDoeSupply
            | Self::UpdateDoeHeader
            | Self::ExportDoePdf => "doe",
            Self::SearchProducts | Self::GetProduct => "catalog",
            Self::ListQuotes | Self::GetQuote => "quote",
            Self::ListInvoices | Self::GetInvoice => "invoice",
        }
    }

    /// Whether the tool works inside one organization and therefore takes
    /// the `organization` argument.
    pub fn is_organization_scoped(self) -> bool {
        !matches!(self, Self::ListOrganizations)
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTool(pub String);

impl fmt::Display for UnknownTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown tool `{}`", self.0)
    }
}

impl std::error::Error for UnknownTool {}

impl FromStr for Tool {
    type Err = UnknownTool;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Tool::ALL
            .into_iter()
            .find(|tool| tool.name() == name)
            .ok_or_else(|| UnknownTool(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_round_trips_through_its_name() {
        for tool in Tool::ALL {
            assert_eq!(tool.name().parse::<Tool>(), Ok(tool));
        }
    }

    #[test]
    fn an_unknown_name_is_refused() {
        assert_eq!(
            "delete_project".parse::<Tool>(),
            Err(UnknownTool("delete_project".to_owned()))
        );
    }

    #[test]
    fn only_doe_tools_write() {
        for tool in Tool::ALL {
            if tool.access() == Access::Write {
                assert_eq!(tool.domain(), "doe", "{tool}");
            }
        }
    }
}

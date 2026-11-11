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
/// OAuth scope granting the maintenance write tool (contract drafts).
pub const SCOPE_MAINTENANCE_WRITE: &str = "mestier.maintenance.write";

/// OAuth scope granting the customer and deal write tools.
pub const SCOPE_CRM_WRITE: &str = "mestier.crm.write";
/// OAuth scope granting the quote write tools.
pub const SCOPE_QUOTE_WRITE: &str = "mestier.quote.write";
/// OAuth scope granting the project and task write tools.
pub const SCOPE_PLANNING_WRITE: &str = "mestier.planning.write";
/// OAuth scope granting the catalogue and supplier write tools.
pub const SCOPE_REFERENCE_WRITE: &str = "mestier.reference.write";
/// OAuth scope granting the knowledge base write tools (notes, tags, links).
pub const SCOPE_KNOWLEDGE_WRITE: &str = "mestier.knowledge.write";

/// Every scope the gateway advertises in its protected resource metadata.
pub const SCOPES_SUPPORTED: [&str; 8] = [
    SCOPE_READ,
    SCOPE_DOE_WRITE,
    SCOPE_MAINTENANCE_WRITE,
    SCOPE_CRM_WRITE,
    SCOPE_QUOTE_WRITE,
    SCOPE_PLANNING_WRITE,
    SCOPE_REFERENCE_WRITE,
    SCOPE_KNOWLEDGE_WRITE,
];

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
    UpdateDoeSupply,
    DeleteDoeSupply,
    AddDoeSection,
    UpdateDoeSection,
    DeleteDoeSection,
    ReorderDoeSections,
    AddDoeFile,
    UpdateDoeFile,
    DeleteDoeFile,
    UpdateDoeHeader,
    UpdateDoeParty,
    ExportDoePdf,
    ListMaintenanceDue,
    GetMaintenanceContract,
    CreateMaintenanceContractDraft,
    ListCustomers,
    GetCustomer,
    CreateCustomer,
    UpdateCustomer,
    CreateCustomerContact,
    UpdateCustomerContact,
    ListDeals,
    GetDeal,
    CreateDeal,
    UpdateDeal,
    MoveDealStage,
    CreateQuote,
    UpdateQuote,
    SetQuoteStatus,
    CreateQuoteRevision,
    ExportQuotePdf,
    ListTasks,
    GetTask,
    GetPlanning,
    CreateProject,
    UpdateProject,
    CreateTask,
    UpdateTask,
    AddTaskComment,
    CreateProduct,
    UpdateProduct,
    ListSuppliers,
    CreateSupplier,
    UpdateSupplier,
    ListSupplierInvoices,
    ListExpenses,
    ListOutstandingInvoices,
    GetProfitability,
    GetWorkedHours,
    SearchPhotoLibrary,
    TagLibraryPhotos,
    PlaceDoeFiles,
    SearchKnowledge,
    GetKnowledgeArticle,
    CreateKnowledgeNote,
    UpdateKnowledgeArticle,
    LinkKnowledgeArticle,
    ListKnowledgeFolder,
    CreateKnowledgeFolder,
    MoveKnowledgeArticles,
    ListTables,
    GetTable,
    ListTableRows,
    CreateTableRow,
    UpdateTableRow,
}

impl Tool {
    pub const ALL: [Tool; 80] = [
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
        Tool::UpdateDoeSupply,
        Tool::DeleteDoeSupply,
        Tool::AddDoeSection,
        Tool::UpdateDoeSection,
        Tool::DeleteDoeSection,
        Tool::ReorderDoeSections,
        Tool::AddDoeFile,
        Tool::UpdateDoeFile,
        Tool::DeleteDoeFile,
        Tool::UpdateDoeHeader,
        Tool::UpdateDoeParty,
        Tool::ExportDoePdf,
        Tool::ListMaintenanceDue,
        Tool::GetMaintenanceContract,
        Tool::CreateMaintenanceContractDraft,
        Tool::ListCustomers,
        Tool::GetCustomer,
        Tool::CreateCustomer,
        Tool::UpdateCustomer,
        Tool::CreateCustomerContact,
        Tool::UpdateCustomerContact,
        Tool::ListDeals,
        Tool::GetDeal,
        Tool::CreateDeal,
        Tool::UpdateDeal,
        Tool::MoveDealStage,
        Tool::CreateQuote,
        Tool::UpdateQuote,
        Tool::SetQuoteStatus,
        Tool::CreateQuoteRevision,
        Tool::ExportQuotePdf,
        Tool::ListTasks,
        Tool::GetTask,
        Tool::GetPlanning,
        Tool::CreateProject,
        Tool::UpdateProject,
        Tool::CreateTask,
        Tool::UpdateTask,
        Tool::AddTaskComment,
        Tool::CreateProduct,
        Tool::UpdateProduct,
        Tool::ListSuppliers,
        Tool::CreateSupplier,
        Tool::UpdateSupplier,
        Tool::ListSupplierInvoices,
        Tool::ListExpenses,
        Tool::ListOutstandingInvoices,
        Tool::GetProfitability,
        Tool::GetWorkedHours,
        Tool::SearchPhotoLibrary,
        Tool::TagLibraryPhotos,
        Tool::PlaceDoeFiles,
        Tool::SearchKnowledge,
        Tool::GetKnowledgeArticle,
        Tool::CreateKnowledgeNote,
        Tool::UpdateKnowledgeArticle,
        Tool::LinkKnowledgeArticle,
        Tool::ListKnowledgeFolder,
        Tool::CreateKnowledgeFolder,
        Tool::MoveKnowledgeArticles,
        Tool::ListTables,
        Tool::GetTable,
        Tool::ListTableRows,
        Tool::CreateTableRow,
        Tool::UpdateTableRow,
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
            Self::UpdateDoeSupply => "update_doe_supply",
            Self::DeleteDoeSupply => "delete_doe_supply",
            Self::AddDoeSection => "add_doe_section",
            Self::UpdateDoeSection => "update_doe_section",
            Self::DeleteDoeSection => "delete_doe_section",
            Self::ReorderDoeSections => "reorder_doe_sections",
            Self::AddDoeFile => "add_doe_file",
            Self::UpdateDoeFile => "update_doe_file",
            Self::DeleteDoeFile => "delete_doe_file",
            Self::UpdateDoeHeader => "update_doe_header",
            Self::UpdateDoeParty => "update_doe_party",
            Self::ExportDoePdf => "export_doe_pdf",
            Self::ListMaintenanceDue => "list_maintenance_due",
            Self::GetMaintenanceContract => "get_maintenance_contract",
            Self::CreateMaintenanceContractDraft => "create_maintenance_contract_draft",
            Self::ListCustomers => "list_customers",
            Self::GetCustomer => "get_customer",
            Self::CreateCustomer => "create_customer",
            Self::UpdateCustomer => "update_customer",
            Self::CreateCustomerContact => "create_customer_contact",
            Self::UpdateCustomerContact => "update_customer_contact",
            Self::ListDeals => "list_deals",
            Self::GetDeal => "get_deal",
            Self::CreateDeal => "create_deal",
            Self::UpdateDeal => "update_deal",
            Self::MoveDealStage => "move_deal_stage",
            Self::CreateQuote => "create_quote",
            Self::UpdateQuote => "update_quote",
            Self::SetQuoteStatus => "set_quote_status",
            Self::CreateQuoteRevision => "create_quote_revision",
            Self::ExportQuotePdf => "export_quote_pdf",
            Self::ListTasks => "list_tasks",
            Self::GetTask => "get_task",
            Self::GetPlanning => "get_planning",
            Self::CreateProject => "create_project",
            Self::UpdateProject => "update_project",
            Self::CreateTask => "create_task",
            Self::UpdateTask => "update_task",
            Self::AddTaskComment => "add_task_comment",
            Self::CreateProduct => "create_product",
            Self::UpdateProduct => "update_product",
            Self::ListSuppliers => "list_suppliers",
            Self::CreateSupplier => "create_supplier",
            Self::UpdateSupplier => "update_supplier",
            Self::ListSupplierInvoices => "list_supplier_invoices",
            Self::ListExpenses => "list_expenses",
            Self::ListOutstandingInvoices => "list_outstanding_invoices",
            Self::GetProfitability => "get_profitability",
            Self::GetWorkedHours => "get_worked_hours",
            Self::SearchPhotoLibrary => "search_photo_library",
            Self::TagLibraryPhotos => "tag_library_photos",
            Self::PlaceDoeFiles => "place_doe_files",
            Self::SearchKnowledge => "search_knowledge",
            Self::GetKnowledgeArticle => "get_knowledge_article",
            Self::CreateKnowledgeNote => "create_knowledge_note",
            Self::UpdateKnowledgeArticle => "update_knowledge_article",
            Self::LinkKnowledgeArticle => "link_knowledge_article",
            Self::ListKnowledgeFolder => "list_knowledge_folder",
            Self::CreateKnowledgeFolder => "create_knowledge_folder",
            Self::MoveKnowledgeArticles => "move_knowledge_articles",
            Self::ListTables => "list_tables",
            Self::GetTable => "get_table",
            Self::ListTableRows => "list_table_rows",
            Self::CreateTableRow => "create_table_row",
            Self::UpdateTableRow => "update_table_row",
        }
    }

    /// No wildcard arm: a new tool has to be classified here, because an
    /// unclassified write would pass for a read.
    pub fn access(self) -> Access {
        match self {
            Self::CreateDoe
            | Self::AddDoeSupplyFromProduct
            | Self::AddDoeSupply
            | Self::UpdateDoeSupply
            | Self::DeleteDoeSupply
            | Self::AddDoeSection
            | Self::UpdateDoeSection
            | Self::DeleteDoeSection
            | Self::ReorderDoeSections
            | Self::AddDoeFile
            | Self::UpdateDoeFile
            | Self::DeleteDoeFile
            | Self::UpdateDoeHeader
            | Self::UpdateDoeParty
            | Self::CreateMaintenanceContractDraft
            | Self::CreateCustomer
            | Self::UpdateCustomer
            | Self::CreateCustomerContact
            | Self::UpdateCustomerContact
            | Self::CreateDeal
            | Self::UpdateDeal
            | Self::MoveDealStage
            | Self::CreateQuote
            | Self::UpdateQuote
            | Self::SetQuoteStatus
            | Self::CreateQuoteRevision
            | Self::CreateProject
            | Self::UpdateProject
            | Self::CreateTask
            | Self::UpdateTask
            | Self::AddTaskComment
            | Self::CreateProduct
            | Self::UpdateProduct
            | Self::CreateSupplier
            | Self::UpdateSupplier
            | Self::TagLibraryPhotos
            | Self::PlaceDoeFiles
            | Self::CreateKnowledgeNote
            | Self::UpdateKnowledgeArticle
            | Self::LinkKnowledgeArticle
            | Self::CreateKnowledgeFolder
            | Self::MoveKnowledgeArticles
            | Self::CreateTableRow
            | Self::UpdateTableRow => Access::Write,
            Self::ListOrganizations
            | Self::ListProjects
            | Self::GetProject
            | Self::ListDoes
            | Self::GetDoe
            | Self::ListDoeModels
            | Self::SearchProducts
            | Self::GetProduct
            | Self::ListQuotes
            | Self::GetQuote
            | Self::ListInvoices
            | Self::GetInvoice
            | Self::ExportDoePdf
            | Self::ListMaintenanceDue
            | Self::GetMaintenanceContract
            | Self::ListCustomers
            | Self::GetCustomer
            | Self::ExportQuotePdf
            | Self::ListDeals
            | Self::GetDeal
            | Self::ListTasks
            | Self::GetTask
            | Self::GetPlanning
            | Self::ListSuppliers
            | Self::ListSupplierInvoices
            | Self::ListExpenses
            | Self::ListOutstandingInvoices
            | Self::GetProfitability
            | Self::GetWorkedHours
            | Self::SearchPhotoLibrary
            | Self::SearchKnowledge
            | Self::GetKnowledgeArticle
            | Self::ListKnowledgeFolder
            | Self::ListTables
            | Self::GetTable
            | Self::ListTableRows => Access::Read,
        }
    }

    /// The business area, as the policy sees it.
    pub fn domain(self) -> &'static str {
        match self {
            Self::ListOrganizations => "organization",
            Self::ListProjects | Self::GetProject => "project",
            Self::CreateProduct | Self::UpdateProduct => "reference",
            Self::ListSuppliers
            | Self::CreateSupplier
            | Self::UpdateSupplier
            | Self::ListSupplierInvoices => "purchase",
            Self::ListExpenses => "expense",
            Self::ListOutstandingInvoices => "invoice",
            Self::GetProfitability | Self::GetWorkedHours => "reporting",
            Self::ListTasks
            | Self::GetTask
            | Self::GetPlanning
            | Self::CreateProject
            | Self::UpdateProject
            | Self::CreateTask
            | Self::UpdateTask
            | Self::AddTaskComment => "planning",
            Self::ListDoes
            | Self::GetDoe
            | Self::ListDoeModels
            | Self::CreateDoe
            | Self::AddDoeSupplyFromProduct
            | Self::AddDoeSupply
            | Self::UpdateDoeSupply
            | Self::DeleteDoeSupply
            | Self::AddDoeSection
            | Self::UpdateDoeSection
            | Self::DeleteDoeSection
            | Self::ReorderDoeSections
            | Self::AddDoeFile
            | Self::UpdateDoeFile
            | Self::DeleteDoeFile
            | Self::UpdateDoeHeader
            | Self::UpdateDoeParty
            | Self::ExportDoePdf
            | Self::SearchPhotoLibrary
            | Self::TagLibraryPhotos
            | Self::PlaceDoeFiles => "doe",
            Self::SearchKnowledge
            | Self::GetKnowledgeArticle
            | Self::CreateKnowledgeNote
            | Self::UpdateKnowledgeArticle
            | Self::LinkKnowledgeArticle
            | Self::ListKnowledgeFolder
            | Self::CreateKnowledgeFolder
            | Self::MoveKnowledgeArticles
            | Self::ListTables
            | Self::GetTable
            | Self::ListTableRows
            | Self::CreateTableRow
            | Self::UpdateTableRow => "knowledge",
            Self::SearchProducts | Self::GetProduct => "catalog",
            Self::ListQuotes
            | Self::GetQuote
            | Self::CreateQuote
            | Self::UpdateQuote
            | Self::SetQuoteStatus
            | Self::CreateQuoteRevision
            | Self::ExportQuotePdf => "quote",
            Self::ListInvoices | Self::GetInvoice => "invoice",
            Self::ListMaintenanceDue
            | Self::GetMaintenanceContract
            | Self::CreateMaintenanceContractDraft => "maintenance",
            Self::ListCustomers
            | Self::GetCustomer
            | Self::CreateCustomer
            | Self::UpdateCustomer
            | Self::CreateCustomerContact
            | Self::UpdateCustomerContact
            | Self::ListDeals
            | Self::GetDeal
            | Self::CreateDeal
            | Self::UpdateDeal
            | Self::MoveDealStage => "crm",
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
    fn only_the_write_domains_write() {
        for tool in Tool::ALL {
            if tool.access() == Access::Write {
                assert!(
                    [
                        "doe",
                        "maintenance",
                        "crm",
                        "quote",
                        "planning",
                        "reference",
                        "purchase",
                        "knowledge"
                    ]
                    .contains(&tool.domain()),
                    "{tool}"
                );
            }
        }
    }

    #[test]
    fn the_write_tools_are_exactly_these() {
        let mut writes: Vec<&str> = Tool::ALL
            .into_iter()
            .filter(|tool| tool.access() == Access::Write)
            .map(Tool::name)
            .collect();
        writes.sort_unstable();

        assert_eq!(
            writes,
            [
                "add_doe_file",
                "add_doe_section",
                "add_doe_supply",
                "add_doe_supply_from_product",
                "add_task_comment",
                "create_customer",
                "create_customer_contact",
                "create_deal",
                "create_doe",
                "create_knowledge_folder",
                "create_knowledge_note",
                "create_maintenance_contract_draft",
                "create_product",
                "create_project",
                "create_quote",
                "create_quote_revision",
                "create_supplier",
                "create_table_row",
                "create_task",
                "delete_doe_file",
                "delete_doe_section",
                "delete_doe_supply",
                "link_knowledge_article",
                "move_deal_stage",
                "move_knowledge_articles",
                "place_doe_files",
                "reorder_doe_sections",
                "set_quote_status",
                "tag_library_photos",
                "update_customer",
                "update_customer_contact",
                "update_deal",
                "update_doe_file",
                "update_doe_header",
                "update_doe_party",
                "update_doe_section",
                "update_doe_supply",
                "update_knowledge_article",
                "update_product",
                "update_project",
                "update_quote",
                "update_supplier",
                "update_table_row",
                "update_task",
            ]
        );
    }

    #[test]
    fn the_maintenance_tools_are_classified() {
        let expected = [
            (Tool::ListMaintenanceDue, Access::Read),
            (Tool::GetMaintenanceContract, Access::Read),
            (Tool::CreateMaintenanceContractDraft, Access::Write),
        ];
        for (tool, access) in expected {
            assert_eq!(tool.access(), access, "{tool}");
            assert_eq!(tool.domain(), "maintenance", "{tool}");
            assert!(tool.is_organization_scoped(), "{tool}");
        }
    }

    #[test]
    fn nothing_activates_validates_or_deletes_maintenance() {
        for tool in Tool::ALL {
            let name = tool.name();
            if tool.domain() == "maintenance" {
                for forbidden in ["activate", "validate", "reject", "delete", "archive"] {
                    assert!(!name.contains(forbidden), "{name}");
                }
            }
        }
    }

    #[test]
    fn the_maintenance_write_scope_is_advertised_with_the_others() {
        assert_eq!(SCOPE_MAINTENANCE_WRITE, "mestier.maintenance.write");
        assert_eq!(
            SCOPES_SUPPORTED,
            [
                "mestier.read",
                "mestier.doe.write",
                "mestier.maintenance.write",
                "mestier.crm.write",
                "mestier.quote.write",
                "mestier.planning.write",
                "mestier.reference.write",
                "mestier.knowledge.write"
            ]
        );
    }

    #[test]
    fn nothing_deletes_issues_or_sends_in_the_crm() {
        for tool in Tool::ALL {
            let name = tool.name();
            if tool.domain() == "crm" {
                for forbidden in ["delete", "archive", "issue", "send"] {
                    assert!(!name.contains(forbidden), "{name}");
                }
                assert!(tool.is_organization_scoped(), "{name}");
            }
        }
    }

    #[test]
    fn the_crm_write_scope_is_advertised() {
        assert_eq!(SCOPE_CRM_WRITE, "mestier.crm.write");
        assert!(SCOPES_SUPPORTED.contains(&SCOPE_CRM_WRITE));
    }

    #[test]
    fn the_quote_tools_cannot_delete_accept_or_issue() {
        for tool in Tool::ALL {
            let name = tool.name();
            if tool.domain() == "quote" {
                for forbidden in ["delete", "accept", "invoice", "issue"] {
                    assert!(!name.contains(forbidden), "{name}");
                }
            }
        }
        assert_eq!(Tool::ExportQuotePdf.access(), Access::Read);
        assert_eq!(SCOPE_QUOTE_WRITE, "mestier.quote.write");
    }

    #[test]
    fn the_planning_tools_never_delete() {
        for tool in Tool::ALL {
            if tool.domain() == "planning" {
                assert!(!tool.name().contains("delete"), "{tool}");
                assert!(tool.is_organization_scoped(), "{tool}");
            }
        }
        assert_eq!(Tool::GetPlanning.access(), Access::Read);
        assert_eq!(SCOPE_PLANNING_WRITE, "mestier.planning.write");
    }

    #[test]
    fn no_write_tool_outside_the_doe_deletes_archives_issues_or_sends() {
        for tool in Tool::ALL {
            if tool.access() == Access::Write && tool.domain() != "doe" {
                for forbidden in ["delete", "archive", "issue", "send", "cancel"] {
                    assert!(!tool.name().contains(forbidden), "{tool}");
                }
            }
        }
    }

    #[test]
    fn the_reference_write_scope_covers_the_catalogue_and_the_suppliers() {
        assert_eq!(SCOPE_REFERENCE_WRITE, "mestier.reference.write");
        assert_eq!(SCOPES_SUPPORTED.len(), 8);
        for tool in [Tool::CreateProduct, Tool::CreateSupplier] {
            assert!(["reference", "purchase"].contains(&tool.domain()));
        }
        for tool in [
            Tool::GetProfitability,
            Tool::GetWorkedHours,
            Tool::ListExpenses,
        ] {
            assert_eq!(tool.access(), Access::Read, "{tool}");
        }
    }

    #[test]
    fn the_photo_library_tools_live_in_the_doe_domain_and_never_delete() {
        let expected = [
            (Tool::SearchPhotoLibrary, Access::Read),
            (Tool::TagLibraryPhotos, Access::Write),
            (Tool::PlaceDoeFiles, Access::Write),
        ];
        for (tool, access) in expected {
            assert_eq!(tool.access(), access, "{tool}");
            assert_eq!(tool.domain(), "doe", "{tool}");
            assert!(tool.is_organization_scoped(), "{tool}");
        }
        for tool in Tool::ALL {
            let name = tool.name();
            if name.contains("photo_library") || name.contains("library_photos") {
                assert!(!name.contains("delete"), "{name}");
            }
        }
    }

    #[test]
    fn the_knowledge_tools_read_write_notes_and_links_but_never_delete() {
        let expected = [
            (Tool::SearchKnowledge, Access::Read),
            (Tool::GetKnowledgeArticle, Access::Read),
            (Tool::CreateKnowledgeNote, Access::Write),
            (Tool::UpdateKnowledgeArticle, Access::Write),
            (Tool::LinkKnowledgeArticle, Access::Write),
            (Tool::ListKnowledgeFolder, Access::Read),
            (Tool::CreateKnowledgeFolder, Access::Write),
            (Tool::MoveKnowledgeArticles, Access::Write),
        ];
        for (tool, access) in expected {
            assert_eq!(tool.access(), access, "{tool}");
            assert_eq!(tool.domain(), "knowledge", "{tool}");
            assert!(tool.is_organization_scoped(), "{tool}");
        }
        for tool in Tool::ALL {
            if tool.domain() == "knowledge" {
                for forbidden in ["delete", "unlink", "upload"] {
                    assert!(!tool.name().contains(forbidden), "{tool}");
                }
            }
        }
        assert_eq!(SCOPE_KNOWLEDGE_WRITE, "mestier.knowledge.write");
    }

    #[test]
    fn the_table_tools_read_and_write_rows_but_never_delete_or_change_the_structure() {
        let expected = [
            (Tool::ListTables, Access::Read),
            (Tool::GetTable, Access::Read),
            (Tool::ListTableRows, Access::Read),
            (Tool::CreateTableRow, Access::Write),
            (Tool::UpdateTableRow, Access::Write),
        ];
        for (tool, access) in expected {
            assert_eq!(tool.access(), access, "{tool}");
            assert_eq!(tool.domain(), "knowledge", "{tool}");
            assert!(tool.is_organization_scoped(), "{tool}");
        }
        let names: Vec<&str> = Tool::ALL
            .into_iter()
            .map(Tool::name)
            .filter(|name| name.contains("table"))
            .collect();
        assert_eq!(
            names,
            [
                "list_tables",
                "get_table",
                "list_table_rows",
                "create_table_row",
                "update_table_row"
            ]
        );
        for name in names {
            for forbidden in ["delete", "remove", "column", "view", "import", "rename"] {
                assert!(!name.contains(forbidden), "{name}");
            }
        }
    }
}

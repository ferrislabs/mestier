use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use super::{require, require_resource};
use crate::{
    KnowledgeKind, OrganizationId,
    application::MestierUseCase,
    domain::{
        customer::ports::CustomerRepository,
        knowledge::{
            ports::KnowledgeArticleRepository,
            tables::{
                CellReference, CreateKnowledgeTableCommand, CreateTableColumnCommand,
                CreateTableRowCommand, CreateTableViewCommand, DeleteKnowledgeTableCommand,
                DeleteTableColumnCommand, DeleteTableRowsCommand, DeleteTableViewCommand,
                ExportTableViewCommand, GetKnowledgeTableCommand, GetTableRowCommand,
                ImportKnowledgeTableCommand, ImportTableRowsCommand, KnowledgeTable,
                ListKnowledgeTablesCommand, ListTableRowsCommand, ReorderTableColumnsCommand,
                ResolvedRow, RowPage, TableColumn, TableOutcome, TableSchema, TableView,
                UpdateKnowledgeTableCommand, UpdateTableColumnCommand, UpdateTableRowCommand,
                UpdateTableViewCommand,
                csv::ImportReport,
                import::{ImportedTable, import_into_table, import_new_table},
                ports::CellReferenceCheck,
                rows::{KnowledgeTableRowService, TableExport},
                service::KnowledgeTableService,
            },
        },
        member::ports::MemberRepository,
        project::ports::ProjectRepository,
        quote::ports::QuoteRepository,
    },
};

struct References<P, C, Q, A, M> {
    projects: P,
    customers: C,
    quotes: Q,
    articles: A,
    members: M,
}

impl<P, C, Q, A, M> CellReferenceCheck for References<P, C, Q, A, M>
where
    P: ProjectRepository,
    C: CustomerRepository,
    Q: QuoteRepository,
    A: KnowledgeArticleRepository,
    M: MemberRepository,
{
    async fn exists(
        &mut self,
        organization_id: OrganizationId,
        reference: CellReference,
    ) -> Result<bool, CoreError> {
        match reference {
            CellReference::Resource(resource) => match require_resource(
                organization_id,
                resource,
                &mut self.projects,
                &mut self.customers,
                &mut self.quotes,
            )
            .await
            {
                Ok(()) => Ok(true),
                Err(CoreError::NotFound) => Ok(false),
                Err(error) => Err(error),
            },
            CellReference::File(id) => Ok(self
                .articles
                .find_by_id(organization_id, id)
                .await?
                .is_some_and(|article| article.kind == KnowledgeKind::File)),
            CellReference::Member(user_id) => Ok(self
                .members
                .find_by_org_and_user(organization_id, user_id)
                .await?
                .is_some()),
        }
    }
}

impl MestierUseCase {
    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn create_knowledge_table(
        &self,
        command: CreateKnowledgeTableCommand,
    ) -> Result<TableOutcome<TableSchema>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .create_table(command, Utc::now())
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn list_knowledge_tables(
        &self,
        command: ListKnowledgeTablesCommand,
    ) -> Result<Vec<KnowledgeTable>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .list(command.organization_id)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn get_knowledge_table(
        &self,
        command: GetKnowledgeTableCommand,
    ) -> Result<TableSchema, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .schema(command.organization_id, command.id)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn update_knowledge_table(
        &self,
        command: UpdateKnowledgeTableCommand,
    ) -> Result<TableOutcome<KnowledgeTable>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .update_table(command, Utc::now())
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn delete_knowledge_table(
        &self,
        command: DeleteKnowledgeTableCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .delete_table(command.organization_id, command.id)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn create_table_column(
        &self,
        command: CreateTableColumnCommand,
    ) -> Result<TableOutcome<TableColumn>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .create_column(command, Utc::now())
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn update_table_column(
        &self,
        command: UpdateTableColumnCommand,
    ) -> Result<TableOutcome<TableColumn>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .update_column(command)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn delete_table_column(
        &self,
        command: DeleteTableColumnCommand,
    ) -> Result<TableOutcome<()>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .delete_column(command.organization_id, command.table_id, command.id)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn reorder_table_columns(
        &self,
        command: ReorderTableColumnsCommand,
    ) -> Result<TableOutcome<Vec<TableColumn>>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .reorder_columns(command)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn create_table_view(
        &self,
        command: CreateTableViewCommand,
    ) -> Result<TableOutcome<TableView>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .create_view(command, Utc::now())
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn update_table_view(
        &self,
        command: UpdateTableViewCommand,
    ) -> Result<TableOutcome<TableView>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .update_view(command, Utc::now())
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz)]
    pub async fn delete_table_view(
        &self,
        command: DeleteTableViewCommand,
    ) -> Result<TableOutcome<()>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableService::new(knowledge_table_repository, knowledge_table_row_repository)
            .delete_view(command.organization_id, command.table_id, command.id)
            .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz, emitter)]
    pub async fn list_table_rows(
        &self,
        command: ListTableRowsCommand,
    ) -> Result<TableOutcome<RowPage>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .list_rows(command)
        .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz, emitter)]
    pub async fn get_table_row(
        &self,
        command: GetTableRowCommand,
    ) -> Result<ResolvedRow, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .get_row(command.organization_id, command.table_id, command.id)
        .await
    }

    #[transactional(
        knowledge_table,
        knowledge_table_row,
        knowledge_article,
        project,
        customer,
        quote,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn create_table_row(
        &self,
        command: CreateTableRowCommand,
    ) -> Result<TableOutcome<ResolvedRow>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut references = References {
            projects: project_repository,
            customers: customer_repository,
            quotes: quote_repository,
            articles: knowledge_article_repository,
            members: member_repository,
        };
        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .create_row(command, &mut references, Utc::now())
        .await
    }

    #[transactional(
        knowledge_table,
        knowledge_table_row,
        knowledge_article,
        project,
        customer,
        quote,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn update_table_row(
        &self,
        command: UpdateTableRowCommand,
    ) -> Result<TableOutcome<ResolvedRow>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut references = References {
            projects: project_repository,
            customers: customer_repository,
            quotes: quote_repository,
            articles: knowledge_article_repository,
            members: member_repository,
        };
        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .update_row(command, &mut references, Utc::now())
        .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz, emitter)]
    pub async fn delete_table_rows(
        &self,
        command: DeleteTableRowsCommand,
    ) -> Result<TableOutcome<u64>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .delete_rows(command.organization_id, command.table_id, &command.ids)
        .await
    }

    #[transactional(
        knowledge_table,
        knowledge_table_row,
        knowledge_article,
        project,
        customer,
        quote,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn import_knowledge_table(
        &self,
        command: ImportKnowledgeTableCommand,
    ) -> Result<TableOutcome<ImportedTable>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut references = References {
            projects: project_repository,
            customers: customer_repository,
            quotes: quote_repository,
            articles: knowledge_article_repository,
            members: member_repository,
        };
        import_new_table(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
            &mut references,
            command,
            Utc::now(),
        )
        .await
    }

    #[transactional(
        knowledge_table,
        knowledge_table_row,
        knowledge_article,
        project,
        customer,
        quote,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn import_table_rows(
        &self,
        command: ImportTableRowsCommand,
    ) -> Result<TableOutcome<ImportReport>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut references = References {
            projects: project_repository,
            customers: customer_repository,
            quotes: quote_repository,
            articles: knowledge_article_repository,
            members: member_repository,
        };
        import_into_table(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
            &mut references,
            command,
            Utc::now(),
        )
        .await
    }

    #[transactional(knowledge_table, knowledge_table_row, member, role, authz, emitter)]
    pub async fn export_table_view(
        &self,
        command: ExportTableViewCommand,
    ) -> Result<TableOutcome<TableExport>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeTableRowService::new(
            knowledge_table_repository,
            knowledge_table_row_repository,
            emitter,
        )
        .export_view(command.organization_id, command.table_id, command.view_id)
        .await
    }
}

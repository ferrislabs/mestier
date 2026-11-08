//! Projects, tasks and the planning.
//!
//! A task patch is partial on the API side: a key left out changes nothing,
//! so `update_task` sends only what it was given. A project patch replaces
//! the whole project, so `update_project` reads the current one first. Both
//! live under the organization path, which is what scopes them.

use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::{
    api::Fault,
    session::{Paging, uuid},
    tools::{MestierTools, finish, json_result},
};

const PROJECT_KEYS: [&str; 4] = ["name", "customer_id", "customer_context_id", "quote_id"];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Backlog,
    Planned,
    InProgress,
    Done,
    Cancelled,
}

fn put<T: Serialize>(body: &mut Map<String, Value>, key: &str, item: &Option<T>) {
    if let Some(item) = item {
        body.insert(
            key.to_owned(),
            serde_json::to_value(item).unwrap_or(Value::Null),
        );
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListTasksArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Page number, starting at 1.")]
    pub page: Option<u64>,
    #[schemars(description = "Items per page, 1 to 100.")]
    pub per_page: Option<u64>,
    #[schemars(description = "Only the tasks of this project (UUID), subtasks included.")]
    pub project_id: Option<String>,
    #[schemars(description = "Only the tasks in one of these statuses.")]
    pub status: Option<Vec<TaskStatus>>,
    #[schemars(description = "Only the tasks assigned to this member (UUID).")]
    pub assignee_id: Option<String>,
    #[schemars(description = "Only the tasks of this customer (UUID).")]
    pub customer_id: Option<String>,
    #[schemars(description = "Text to look for in the title, case-insensitive.")]
    pub query: Option<String>,
    #[schemars(
        description = "true for the tasks with no start date, false for the scheduled ones."
    )]
    pub unscheduled: Option<bool>,
}

impl ListTasksArgs {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = Vec::new();
        for (key, value) in [
            ("project_id", &self.project_id),
            ("assignee_id", &self.assignee_id),
            ("customer_id", &self.customer_id),
            ("q", &self.query),
        ] {
            if let Some(value) = value {
                query.push((key, value.clone()));
            }
        }
        if let Some(statuses) = &self.status {
            for status in statuses {
                if let Value::String(status) = json!(status) {
                    query.push(("status", status));
                }
            }
        }
        if let Some(flag) = self.unscheduled {
            query.push(("unscheduled", flag.to_string()));
        }
        query
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct TaskArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the task.")]
    pub task_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct PlanningArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "First day of the window, YYYY-MM-DD.")]
    pub from: String,
    #[schemars(description = "Last day of the window, YYYY-MM-DD.")]
    pub to: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateProjectArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    pub name: String,
    #[schemars(description = "UUID of the customer. Leave out for internal work.")]
    pub customer_id: Option<String>,
    #[schemars(description = "UUID of the customer's site or address.")]
    pub customer_context_id: Option<String>,
    #[schemars(description = "UUID of the quote the project comes from.")]
    pub quote_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateProjectArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the project.")]
    pub project_id: String,
    pub name: Option<String>,
    #[schemars(description = "UUID of the customer.")]
    pub customer_id: Option<String>,
    #[schemars(description = "UUID of the customer's site or address.")]
    pub customer_context_id: Option<String>,
    #[schemars(description = "UUID of the quote the project comes from.")]
    pub quote_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateTaskArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    pub title: String,
    pub description: Option<String>,
    #[schemars(description = "Start, RFC 3339, for example 2026-11-09T08:00:00Z.")]
    pub starts_at: Option<String>,
    #[schemars(description = "End, RFC 3339.")]
    pub ends_at: Option<String>,
    pub all_day: Option<bool>,
    #[schemars(description = "BACKLOG for agreed work not yet scheduled. Defaults to PLANNED.")]
    pub status: Option<TaskStatus>,
    #[schemars(
        description = "Whether the task makes its assignees unavailable elsewhere. Always stated, never guessed: ask the user if unsure."
    )]
    pub blocks_availability: bool,
    #[schemars(description = "UUID of the project the task is costed against.")]
    pub project_id: Option<String>,
    #[schemars(description = "UUID of the customer.")]
    pub customer_id: Option<String>,
    #[schemars(description = "UUID of the parent task, to create a subtask.")]
    pub parent_task_id: Option<String>,
}

impl CreateTaskArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("title".to_owned(), json!(self.title));
        body.insert(
            "blocks_availability".to_owned(),
            json!(self.blocks_availability),
        );
        body.insert("all_day".to_owned(), json!(self.all_day.unwrap_or(false)));
        put(&mut body, "description", &self.description);
        put(&mut body, "starts_at", &self.starts_at);
        put(&mut body, "ends_at", &self.ends_at);
        put(&mut body, "status", &self.status);
        put(&mut body, "project_id", &self.project_id);
        put(&mut body, "customer_id", &self.customer_id);
        put(&mut body, "parent_task_id", &self.parent_task_id);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateTaskArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the task.")]
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[schemars(description = "Start, RFC 3339.")]
    pub starts_at: Option<String>,
    #[schemars(description = "End, RFC 3339.")]
    pub ends_at: Option<String>,
    pub all_day: Option<bool>,
    pub status: Option<TaskStatus>,
    pub blocks_availability: Option<bool>,
    #[schemars(
        description = "The complete list of assigned members (UUIDs), replacing the current one. [] unassigns everyone."
    )]
    pub assignee_member_ids: Option<Vec<String>>,
    #[schemars(description = "The complete list of label UUIDs, replacing the current one.")]
    pub label_ids: Option<Vec<String>>,
    #[schemars(description = "UUID of the project to attach the task to.")]
    pub project_id: Option<String>,
}

impl UpdateTaskArgs {
    fn body(&self) -> Result<Value, Fault> {
        let mut body = Map::new();
        put(&mut body, "title", &self.title);
        put(&mut body, "description", &self.description);
        put(&mut body, "starts_at", &self.starts_at);
        put(&mut body, "ends_at", &self.ends_at);
        put(&mut body, "all_day", &self.all_day);
        put(&mut body, "status", &self.status);
        put(&mut body, "blocks_availability", &self.blocks_availability);
        put(&mut body, "label_ids", &self.label_ids);
        put(&mut body, "project_id", &self.project_id);
        if let Some(members) = &self.assignee_member_ids {
            let assignees: Vec<Value> = members
                .iter()
                .map(|member| uuid(member).map(|id| json!({"member_id": id})))
                .collect::<Result<_, _>>()?;
            body.insert("assignees".to_owned(), Value::Array(assignees));
        }
        if body.is_empty() {
            return Err(Fault::new("aucune modification demandée"));
        }
        Ok(Value::Object(body))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct AddCommentArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the task.")]
    pub task_id: String,
    pub body: String,
}

#[tool_router(router = planning_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "List the tasks (planification) of an organization (paginated), optionally by project, status, assignee, customer, title text, or whether they are scheduled. Each has an id usable with get_task."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_tasks"))]
    async fn list_tasks(
        &self,
        Parameters(args): Parameters<ListTasksArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_tasks") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let page = Paging {
                    page: args.page,
                    per_page: args.per_page,
                };
                let body = s
                    .list_filtered(args.organization.as_deref(), "tasks", page, args.query())
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one task by id: dates, status, assignees, labels, project.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_task"))]
    async fn get_task(
        &self,
        Parameters(args): Parameters<TaskArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_task") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_scoped(args.organization.as_deref(), "tasks", &args.task_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Get the planning of an organization over a window of days: the members and their tasks, absences and maintenance visits."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_planning"))]
    async fn get_planning(
        &self,
        Parameters(args): Parameters<PlanningArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_planning") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_in_org(
                        args.organization.as_deref(),
                        "planning",
                        &[("from", args.from.clone()), ("to", args.to.clone())],
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a project, for a customer or for internal work (no customer). Returns the project."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_project"))]
    async fn create_project(
        &self,
        Parameters(args): Parameters<CreateProjectArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_project") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let mut body = Map::new();
                body.insert("name".to_owned(), json!(args.name));
                put(&mut body, "customer_id", &args.customer_id);
                put(&mut body, "customer_context_id", &args.customer_context_id);
                put(&mut body, "quote_id", &args.quote_id);
                let data = s
                    .create_in_org(
                        args.organization.as_deref(),
                        "projects",
                        &Value::Object(body),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a project. Only the given fields change; a link cannot be cleared from here. Returns the project."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_project"))]
    async fn update_project(
        &self,
        Parameters(args): Parameters<UpdateProjectArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_project") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let mut patch = Map::new();
                put(&mut patch, "name", &args.name);
                put(&mut patch, "customer_id", &args.customer_id);
                put(&mut patch, "customer_context_id", &args.customer_context_id);
                put(&mut patch, "quote_id", &args.quote_id);
                let data = s
                    .update_in_org(
                        args.organization.as_deref(),
                        "projects",
                        &args.project_id,
                        &PROJECT_KEYS,
                        patch,
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a task. `blocks_availability` is required: say whether it makes its assignees unavailable. Assign people afterwards with update_task. Returns the task."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_task"))]
    async fn create_task(
        &self,
        Parameters(args): Parameters<CreateTaskArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_task") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_in_org(args.organization.as_deref(), "tasks", &args.body())
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a task: title, description, dates, status, assignees, labels or project. Only the given fields change; assignees and labels are the complete new list. Returns the task."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_task"))]
    async fn update_task(
        &self,
        Parameters(args): Parameters<UpdateTaskArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_task") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let id = uuid(&args.task_id)?;
                let data = s
                    .patch_in_org(
                        args.organization.as_deref(),
                        &format!("tasks/{id}"),
                        &args.body()?,
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(description = "Add a comment to a task. Returns the comment.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "add_task_comment"))]
    async fn add_task_comment(
        &self,
        Parameters(args): Parameters<AddCommentArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "add_task_comment") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let id = uuid(&args.task_id)?;
                let data = s
                    .create_in_org(
                        args.organization.as_deref(),
                        &format!("tasks/{id}/comments"),
                        &json!({"body": args.body}),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }
}

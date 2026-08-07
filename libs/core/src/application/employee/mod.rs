use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Employee, EmployeeId, OrganizationId, UserId,
    application::MestierUseCase,
    domain::employee::{
        commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
        service::EmployeeService,
    },
};

impl MestierUseCase {
    #[transactional(employee)]
    pub async fn create_employee(
        &self,
        command: CreateEmployeeCommand,
    ) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.create_employee(command).await
    }

    #[transactional(employee)]
    pub async fn get_employee(&self, id: EmployeeId) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.get_employee(id).await
    }

    #[transactional(employee)]
    pub async fn find_employee_by_user_id(
        &self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Employee>, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service
            .find_employee_by_user_id(organization_id, user_id)
            .await
    }

    #[transactional(employee)]
    pub async fn list_employees(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Employee>, u64), CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.list_employees(organization_id, limit, offset).await
    }

    #[transactional(employee)]
    pub async fn update_employee(
        &self,
        command: UpdateEmployeeCommand,
    ) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.update_employee(command).await
    }

    #[transactional(employee)]
    pub async fn link_employee_user(
        &self,
        command: LinkEmployeeUserCommand,
    ) -> Result<Employee, CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.link_employee_user(command).await
    }

    #[transactional(employee)]
    pub async fn soft_delete_employee(&self, id: EmployeeId) -> Result<(), CoreError> {
        let mut service = EmployeeService::new(employee_repository);
        service.soft_delete_employee(id).await
    }
}

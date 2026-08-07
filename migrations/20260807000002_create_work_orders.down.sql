DROP INDEX IF EXISTS idx_assignments_employee_id;
DROP INDEX IF EXISTS idx_assignments_work_order_id;
DROP INDEX IF EXISTS idx_assignments_org_id;
DROP TABLE IF EXISTS assignments;

DROP INDEX IF EXISTS idx_work_orders_deleted_at;
DROP INDEX IF EXISTS idx_work_orders_quote_id;
DROP INDEX IF EXISTS idx_work_orders_customer_context_id;
DROP INDEX IF EXISTS idx_work_orders_customer_id;
DROP INDEX IF EXISTS idx_work_orders_org_id_starts_at;
DROP TABLE IF EXISTS work_orders;

DROP TYPE IF EXISTS work_order_status;

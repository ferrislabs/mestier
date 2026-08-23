DROP INDEX IF EXISTS idx_employee_cost_bases_org_id;
DROP INDEX IF EXISTS idx_employee_cost_bases_employee_id_effective_from;
DROP TABLE IF EXISTS employee_cost_bases;

-- btree_gist is left in place: other objects may come to depend on it, and
-- dropping a shared extension is not this migration's call to make.

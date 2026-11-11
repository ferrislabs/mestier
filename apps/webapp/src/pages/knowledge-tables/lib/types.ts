import type { Schemas } from '#/api/api.client'

export type ColumnKind = Schemas.ColumnKind
export type ColumnConfig = Schemas.ColumnConfig
export type FilterOperator = Schemas.FilterOperator
export type SelectColor = Schemas.SelectColor
export type LinkKind = Schemas.KnowledgeResourceType
export type ViewKind = Schemas.ViewKind
export type ViewConfig = Schemas.ViewConfig
export type RowFilter = Schemas.RowFilter
export type RowSort = Schemas.RowSort
export type SortDirection = Schemas.SortDirection
export type KnowledgeTable = Schemas.KnowledgeTableResponse
export type KnowledgeTableDetail = Schemas.KnowledgeTableDetailResponse
export type TableColumn = Schemas.TableColumnResponse
export type TableView = Schemas.TableViewResponse
export type TableRow = Schemas.TableRowResponse
export type TableRowPage = Schemas.TableRowPageResponse

export interface SelectOption {
	id: string
	label: string
	color: SelectColor
}

export interface CellLabel {
	label: string | null
	missing: boolean
}

export interface FilterCondition {
	column_id: string
	operator: FilterOperator
	value: unknown
}

export interface FilterDraft extends FilterCondition {
	key: string
}

export interface LinkValue {
	kind: LinkKind
	id: string
}

export type RowValues = Record<string, unknown>

export interface Choice {
	id: string
	label: string
}

import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'

/**
 * The three resolutions, no synthetic "all" option: the backend list has no
 * "every state" mode (see `use-assignment-reports.ts`'s own doc — an absent
 * filter defaults to `PENDING`), so a filter value here is always one real
 * request, never a client-side narrowing of a broader fetch.
 */
export const ASSIGNMENT_REPORT_FILTER_OPTIONS: DataViewOption[] = [
	{ value: 'PENDING', label: 'En attente' },
	{ value: 'APPLIED', label: 'Appliqués' },
	{ value: 'DISMISSED', label: 'Rejetés' },
]

export const ASSIGNMENT_REPORT_SORT_OPTIONS: DataViewOption[] = [
	{ value: 'created-desc', label: 'Plus récents' },
	{ value: 'created-asc', label: 'Plus anciens' },
]

export const DEFAULT_ASSIGNMENT_REPORT_LIST_STATE: DataViewUrlState = {
	search: '',
	filter: 'PENDING',
	sort: 'created-desc',
	page: 1,
	pageSize: 25,
}

export function getAssignmentReportListUrlState(): DataViewUrlState {
	return readDataViewUrlState({
		defaults: DEFAULT_ASSIGNMENT_REPORT_LIST_STATE,
		isValidFilter: isValidAssignmentReportFilter,
		isValidSort: isValidAssignmentReportSortValue,
	})
}

export function writeAssignmentReportListUrlState(state: DataViewUrlState) {
	writeDataViewUrlState(state, DEFAULT_ASSIGNMENT_REPORT_LIST_STATE)
}

export function isValidAssignmentReportFilter(value: string): boolean {
	return ASSIGNMENT_REPORT_FILTER_OPTIONS.some(
		(option) => option.value === value,
	)
}

export function isValidAssignmentReportSortValue(value: string): boolean {
	return ASSIGNMENT_REPORT_SORT_OPTIONS.some((option) => option.value === value)
}

import type { Project } from '#/hooks/use-projects'
import type { MissionStats } from '#/pages/projects/lib/missions'
import type {
	Person,
	ProjectExtras,
	ProjectStatus,
} from '#/pages/projects/lib/model'
import { countLabel, normalizeText } from '#/pages/projects/lib/text'

export type StatusFilter = 'actifs' | 'archives' | 'tous'

export const CLIENT_ALL = 'tous'

export const CLIENT_INTERNAL = 'interne'

export interface ProjectFilters {
	query: string
	status: StatusFilter
	client: string
}

export const DEFAULT_FILTERS: ProjectFilters = {
	query: '',
	status: 'actifs',
	client: CLIENT_ALL,
}

export const STATUS_OPTIONS: readonly { value: StatusFilter; label: string }[] =
	[
		{ value: 'actifs', label: 'Actifs' },
		{ value: 'archives', label: 'Archivés' },
		{ value: 'tous', label: 'Tous' },
	]

export function isStatusFilter(value: unknown): value is StatusFilter {
	return STATUS_OPTIONS.some((option) => option.value === value)
}

export function statusOf(project: Pick<Project, 'archived_at'>): ProjectStatus {
	return project.archived_at ? 'archived' : 'active'
}

export interface ProjectRow {
	project: Project
	status: ProjectStatus
	customerName: string | null
	extras: ProjectExtras
	missions: MissionStats
	doeRevision: string | null
	owner: Person | undefined
}

export function resolveClient(
	client: string,
	customers: readonly { id: string }[],
): string {
	if (client === CLIENT_ALL || client === CLIENT_INTERNAL) return client
	return customers.some((customer) => customer.id === client)
		? client
		: CLIENT_ALL
}

function matchesStatus(row: ProjectRow, status: StatusFilter): boolean {
	if (status === 'tous') return true
	return row.status === (status === 'actifs' ? 'active' : 'archived')
}

function matchesClient(row: ProjectRow, client: string): boolean {
	if (client === CLIENT_ALL) return true
	if (client === CLIENT_INTERNAL) return !row.project.customer_id
	return row.project.customer_id === client
}

function matchesQuery(row: ProjectRow, query: string): boolean {
	if (query === '') return true
	return [row.project.name, row.customerName ?? '', row.extras.place]
		.map(normalizeText)
		.join('\n')
		.includes(query)
}

function compareRows(a: ProjectRow, b: ProjectRow): number {
	return (
		Number(a.status === 'archived') - Number(b.status === 'archived') ||
		a.project.name.localeCompare(b.project.name, 'fr')
	)
}

export function filterRows(
	rows: readonly ProjectRow[],
	filters: ProjectFilters,
): ProjectRow[] {
	const query = normalizeText(filters.query)
	return rows
		.filter(
			(row) =>
				matchesStatus(row, filters.status) &&
				matchesClient(row, filters.client) &&
				matchesQuery(row, query),
		)
		.sort(compareRows)
}

export function hasActiveFilters(filters: ProjectFilters): boolean {
	return (
		filters.query.trim() !== '' ||
		filters.status !== DEFAULT_FILTERS.status ||
		filters.client !== DEFAULT_FILTERS.client
	)
}

export type ListEmpty = 'none' | 'no-active' | 'no-archived' | 'no-match'

export function listEmpty(total: number, filters: ProjectFilters): ListEmpty {
	if (total === 0) return 'none'
	const bare =
		filters.query.trim() === '' && filters.client === DEFAULT_FILTERS.client
	if (bare && filters.status === 'archives') return 'no-archived'
	if (bare && filters.status === 'actifs') return 'no-active'
	return 'no-match'
}

export function projectsSummary(
	rows: readonly { status: ProjectStatus }[],
): string {
	const active = rows.filter((row) => row.status === 'active')
	return `${countLabel(active.length, 'projet actif', 'projets actifs')}, ${countLabel(
		rows.length - active.length,
		'archivé',
		'archivés',
	)}`
}

export interface ProjectTiles {
	active: number
	internal: number
	missionsInProgress: number
	doeToCreate: number
}

export function projectTiles(rows: readonly ProjectRow[]): ProjectTiles {
	const active = rows.filter((row) => row.status === 'active')
	return {
		active: active.length,
		internal: active.filter((row) => !row.project.customer_id).length,
		missionsInProgress: active.reduce(
			(total, row) => total + row.missions.inProgress,
			0,
		),
		doeToCreate: active.filter((row) => row.doeRevision === null).length,
	}
}

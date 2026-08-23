import { AlertCircle, ClipboardList, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import {
	DataViewPagination,
	type DataViewSortOption,
	DataViewToolbar,
	getPaginationViewModel,
	useDataView,
} from '#/components/data-view'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type {
	AssignmentReport,
	AssignmentReportResolution,
	PaginationMetadata,
} from '#/hooks/use-assignment-reports'
import {
	ASSIGNMENT_REPORT_FILTER_OPTIONS,
	ASSIGNMENT_REPORT_SORT_OPTIONS,
} from '#/pages/planning/assignment-report-list-url-state'
import {
	minutesLabel,
	reportedAtLabel,
} from '#/pages/planning/lib/pending-reports'

export interface AssignmentReportListUIProps {
	organizationName: string
	reports: AssignmentReport[]
	pagination?: PaginationMetadata | null
	page: number
	pageSize: number
	resolution: AssignmentReportResolution
	memberName: (memberId: string) => string
	isLoading?: boolean
	error?: string | null
	onRetry?: () => void
	onPageChange: (page: number) => void
	onPageSizeChange: (pageSize: number) => void
	onResolutionChange: (resolution: AssignmentReportResolution) => void
}

const SORT_OPTIONS: DataViewSortOption<AssignmentReport>[] = [
	{
		value: 'created-desc',
		label: 'Plus récents',
		compare: (a, b) => b.created_at.localeCompare(a.created_at),
	},
	{
		value: 'created-asc',
		label: 'Plus anciens',
		compare: (a, b) => a.created_at.localeCompare(b.created_at),
	},
]

function resolutionTone(resolution: AssignmentReportResolution) {
	if (resolution === 'APPLIED') return 'success' as const
	if (resolution === 'DISMISSED') return 'neutral' as const
	return 'warning' as const
}

function resolutionLabel(resolution: AssignmentReportResolution): string {
	return (
		ASSIGNMENT_REPORT_FILTER_OPTIONS.find(
			(option) => option.value === resolution,
		)?.label ?? resolution
	)
}

/**
 * The manager's queue, filterable by resolution — "so a week's reports can
 * be worked through in one pass" (see the issue). Resolving is not done
 * here: each row is a summary, and the arbitration itself lives on the
 * task sheet (`PendingReportPanel`), which is where "what was planned"
 * lives too. This list exists to *find* a report, not to act on it blind.
 *
 * The filter drives a real server request (the backend supports one
 * resolution per call — see `use-assignment-reports.ts`'s own doc); the
 * search box, unlike the filter, only narrows what is already on the
 * current page, since no search exists on the backend endpoint.
 */
export function AssignmentReportListUI({
	organizationName,
	reports,
	pagination,
	page,
	pageSize,
	resolution,
	memberName,
	isLoading,
	error,
	onRetry,
	onPageChange,
	onPageSizeChange,
	onResolutionChange,
}: AssignmentReportListUIProps) {
	const [search, setSearch] = useState('')
	const [sort, setSort] = useState('created-desc')

	const paginationView = getPaginationViewModel(
		pagination,
		reports.length,
		page,
		pageSize,
	)

	const dataView = useDataView({
		data: reports,
		search,
		searchPredicate: (report, query) =>
			memberName(report.reported_by).toLowerCase().includes(query) ||
			(report.comment?.toLowerCase().includes(query) ?? false),
		filter: resolution,
		filterPredicate: () => true,
		defaultFilter: resolution,
		sort,
		sortOptions: SORT_OPTIONS,
		page,
		pageSize,
		manualPagination: true,
		totalCount: paginationView.totalCount,
		pageCount: paginationView.pageCount,
		from: paginationView.from,
		to: paginationView.to,
	})

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Écarts signalés"
				description="Les corrections que les équipes de terrain ont déclarées sur leurs projets."
				actions={
					<Button
						type="button"
						variant="outline"
						onClick={onRetry}
						disabled={!onRetry}
					>
						<RefreshCw />
						Actualiser
					</Button>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</SectionCard>
			) : null}

			<SectionCard>
				<SectionHeader
					title={`${resolutionLabel(resolution)} (${dataView.totalCount})`}
					description="Le filtre interroge le serveur ; la recherche ne porte que sur la page affichée."
				/>
				<div className="border-b p-4">
					<DataViewToolbar
						search={search}
						onSearchChange={setSearch}
						searchPlaceholder="Rechercher sur cette page…"
						filter={resolution}
						onFilterChange={(value) => {
							onResolutionChange(value as AssignmentReportResolution)
							onPageChange(1)
						}}
						filterOptions={ASSIGNMENT_REPORT_FILTER_OPTIONS}
						defaultFilter={resolution}
						sort={sort}
						onSortChange={setSort}
						sortOptions={ASSIGNMENT_REPORT_SORT_OPTIONS}
						filteredCount={dataView.filteredCount}
						totalCount={dataView.totalCount}
						onReset={() => setSearch('')}
					/>
				</div>

				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : dataView.rows.length === 0 ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<ClipboardList className="size-5" />
						</div>
						<p className="font-medium">
							{search
								? 'Aucun écart ne correspond à cette recherche.'
								: `Aucun écart ${resolutionLabel(resolution).toLowerCase()}.`}
						</p>
					</div>
				) : (
					<ul className="divide-y">
						{dataView.rows.map((report) => (
							<li key={report.id} className="px-5 py-4">
								<div className="flex flex-wrap items-center gap-2">
									<StatusBadge tone={resolutionTone(report.resolution)}>
										{resolutionLabel(report.resolution)}
									</StatusBadge>
									<p className="text-sm font-medium">
										{minutesLabel(report.reported_minutes)} déclarées
									</p>
									<p className="text-xs text-muted-foreground">
										par {memberName(report.reported_by)}, le{' '}
										{reportedAtLabel(report.created_at)}
									</p>
								</div>
								{report.comment ? (
									<p className="mt-1.5 text-sm text-muted-foreground">
										« {report.comment} »
									</p>
								) : null}
								{report.resolution !== 'PENDING' ? (
									<p className="mt-1.5 text-xs text-muted-foreground">
										{resolutionLabel(report.resolution)} par{' '}
										{report.resolved_by
											? memberName(report.resolved_by)
											: 'un responsable'}
										{report.resolution_note
											? ` — « ${report.resolution_note} »`
											: null}
									</p>
								) : null}
								<p className="mt-1.5 font-mono text-xs text-muted-foreground">
									Affectation {report.task_assignment_id}
								</p>
							</li>
						))}
					</ul>
				)}

				{!isLoading ? (
					<div className="border-t p-4">
						<DataViewPagination
							page={dataView.page}
							pageCount={dataView.pageCount}
							pageSize={pageSize}
							from={dataView.from}
							to={dataView.to}
							totalCount={dataView.totalCount}
							onPageChange={onPageChange}
							onPageSizeChange={onPageSizeChange}
						/>
					</div>
				) : null}
			</SectionCard>
		</PageShell>
	)
}

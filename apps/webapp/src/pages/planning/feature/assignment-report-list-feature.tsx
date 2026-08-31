import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type AssignmentReportResolution,
	useAssignmentReports,
} from '#/hooks/use-assignment-reports'
import { usePlanning } from '#/hooks/use-planning'
import {
	getAssignmentReportListUrlState,
	writeAssignmentReportListUrlState,
} from '#/pages/planning/assignment-report-list-url-state'
import { memberNamesById } from '#/pages/planning/lib/task-list'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'
import { AssignmentReportListUI } from '#/pages/planning/ui/assignment-report-list-ui'

/**
 * Mounts the correction-loop list screen. Router-agnostic, like
 * `TaskListFeature`: no URL-driven `view`/`date` here, but the filter,
 * page and page size still round-trip through the URL (see
 * `assignment-report-list-url-state.ts`), the same way the quote list does.
 */
export function AssignmentReportListFeature() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Les écarts signalés nécessitent une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<AssignmentReportListScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function AssignmentReportListScreen({
	organizationId,
}: {
	organizationId: string
}) {
	const [initialState] = useState(getAssignmentReportListUrlState)
	const [page, setPage] = useState(initialState.page)
	const [pageSize, setPageSize] = useState(initialState.pageSize)
	const [resolution, setResolution] = useState<AssignmentReportResolution>(
		isAssignmentReportResolution(initialState.filter)
			? initialState.filter
			: 'PENDING',
	)

	const reportsQuery = useAssignmentReports(
		organizationId,
		resolution,
		page,
		pageSize,
	)

	// Fetched solely for the roster — `reported_by`/`resolved_by` on a
	// report are bare member ids, and this is the same resolution used by
	// `TaskListFeature` for the same reason. A fixed "today" window is
	// enough, see that feature's own note.
	const resourcesWindow = computeWindow('day', todayIsoDate())
	const planningQuery = usePlanning(organizationId, resourcesWindow)
	const resources = planningQuery.data?.data.resources ?? []
	const namesById = memberNamesById(resources)

	function memberName(memberId: string): string {
		return namesById[memberId] ?? 'Membre inconnu'
	}

	function updateUrlState(next: {
		page: number
		pageSize: number
		resolution: AssignmentReportResolution
	}) {
		writeAssignmentReportListUrlState({
			search: '',
			filter: next.resolution,
			sort: 'created-desc',
			page: next.page,
			pageSize: next.pageSize,
		})
	}

	return (
		<AssignmentReportListUI
			reports={reportsQuery.data?.data ?? []}
			pagination={reportsQuery.data?.pagination}
			page={page}
			pageSize={pageSize}
			resolution={resolution}
			memberName={memberName}
			isLoading={reportsQuery.isLoading || planningQuery.isLoading}
			error={reportsQuery.error?.message ?? null}
			onRetry={() => void reportsQuery.refetch()}
			onPageChange={(nextPage) => {
				setPage(nextPage)
				updateUrlState({ page: nextPage, pageSize, resolution })
			}}
			onPageSizeChange={(nextPageSize) => {
				setPageSize(nextPageSize)
				setPage(1)
				updateUrlState({ page: 1, pageSize: nextPageSize, resolution })
			}}
			onResolutionChange={(nextResolution) => {
				setResolution(nextResolution)
				setPage(1)
				updateUrlState({ page: 1, pageSize, resolution: nextResolution })
			}}
		/>
	)
}

function isAssignmentReportResolution(
	value: string,
): value is AssignmentReportResolution {
	return value === 'PENDING' || value === 'APPLIED' || value === 'DISMISSED'
}

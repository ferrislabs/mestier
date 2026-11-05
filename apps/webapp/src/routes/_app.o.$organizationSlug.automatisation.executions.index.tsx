import { createFileRoute } from '@tanstack/react-router'
import { ExecutionsLogFeature } from '#/pages/automation/feature/executions-log-feature'
import {
	ALL,
	PERIOD_LABELS,
	type PeriodFilter,
	type StatusFilter,
} from '#/pages/automation/lib/execution-log'

interface ExecutionsSearch {
	statut?: Exclude<StatusFilter, 'all'>
	workflow?: string
	declencheur?: string
	periode?: Exclude<PeriodFilter, 'all'>
}

const STATUSES: readonly string[] = ['running', 'success', 'error']

function text(value: unknown): string | undefined {
	return typeof value === 'string' && value !== '' && value !== ALL
		? value
		: undefined
}

function validateSearch(raw: Record<string, unknown>): ExecutionsSearch {
	const statut = text(raw.statut)
	const periode = text(raw.periode)
	const workflow = text(raw.workflow)
	const declencheur = text(raw.declencheur)
	return {
		...(statut && STATUSES.includes(statut)
			? { statut: statut as ExecutionsSearch['statut'] }
			: {}),
		...(workflow ? { workflow } : {}),
		...(declencheur ? { declencheur } : {}),
		...(periode && periode in PERIOD_LABELS
			? { periode: periode as ExecutionsSearch['periode'] }
			: {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/executions/',
)({
	validateSearch,
	component: ExecutionsRoute,
})

function ExecutionsRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ExecutionsLogFeature
			filters={{
				status: search.statut ?? ALL,
				workflow: search.workflow ?? ALL,
				trigger: search.declencheur ?? ALL,
				period: search.periode ?? ALL,
			}}
			onFiltersChange={(patch) =>
				navigate({
					replace: true,
					search: (previous: ExecutionsSearch) =>
						validateSearch({
							...previous,
							...(patch.status !== undefined ? { statut: patch.status } : {}),
							...(patch.workflow !== undefined
								? { workflow: patch.workflow }
								: {}),
							...(patch.trigger !== undefined
								? { declencheur: patch.trigger }
								: {}),
							...(patch.period !== undefined ? { periode: patch.period } : {}),
						}),
				})
			}
		/>
	)
}

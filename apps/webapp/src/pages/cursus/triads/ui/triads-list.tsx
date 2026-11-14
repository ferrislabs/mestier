import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	type CountQualifier,
	countSummary,
	TRIAD_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'
import type { FilterOption } from '#/pages/cursus/shared/lib/list-filters'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'
import type { TriadSummary } from '#/pages/cursus/triads/feature/use-triads'
import { TriadsTable } from '#/pages/cursus/triads/ui/triads-table'

export type TriadStatusFilter = 'open' | 'closed' | 'all'

const STATUS_OPTIONS: FilterOption<TriadStatusFilter>[] = [
	{ value: 'open', label: 'Ouverts' },
	{ value: 'closed', label: 'Clos' },
	{ value: 'all', label: 'Tous' },
]

const STATUS_QUALIFIER: Record<TriadStatusFilter, CountQualifier | undefined> =
	{
		open: 'open',
		closed: 'closed',
		all: undefined,
	}

const ALL_PROMOTIONS = 'all'

interface TriadsListProps {
	triads: TriadSummary[]
	total: number | undefined
	promotions: SelectOption[]
	promotionId: string | undefined
	status: TriadStatusFilter
	search: string
	isLoading: boolean
	canManage: boolean
	onSearchChange: (search: string) => void
	onPromotionChange: (promotionId: string | undefined) => void
	onStatusChange: (status: TriadStatusFilter) => void
	onResetFilters: () => void
	onOpen: (triadId: string) => void
	onCreate: () => void
}

export function TriadsList({
	triads,
	total,
	promotions,
	promotionId,
	status,
	search,
	isLoading,
	canManage,
	onSearchChange,
	onPromotionChange,
	onStatusChange,
	onResetFilters,
	onOpen,
	onCreate,
}: TriadsListProps) {
	const filtered =
		search.trim() !== '' || status !== 'open' || promotionId !== undefined

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Trinômes"
				description={
					total === undefined
						? undefined
						: countSummary(total, TRIAD_NOUN, STATUS_QUALIFIER[status])
				}
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden /> Nouveau trinôme
						</Button>
					) : null
				}
			/>
			<ListToolbar
				search={search}
				searchPlaceholder="Rechercher un apprenti, une entreprise, une promotion"
				onSearch={onSearchChange}
				status={status}
				statusOptions={STATUS_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={onStatusChange}
				select={{
					value: promotionId ?? ALL_PROMOTIONS,
					label: 'Filtrer par promotion',
					options: [
						{ value: ALL_PROMOTIONS, label: 'Toutes les promotions' },
						...promotions,
					],
					onChange: (next) =>
						onPromotionChange(next === ALL_PROMOTIONS ? undefined : next),
				}}
			/>
			<TriadsTable
				triads={isLoading ? [] : triads}
				showPromotion
				onOpen={onOpen}
				empty={
					isLoading
						? undefined
						: {
								title: 'Aucun trinôme ne correspond.',
								hint: filtered
									? 'Vérifie l’orthographe, le statut ou la promotion sélectionnée.'
									: undefined,
								onReset: filtered ? onResetFilters : undefined,
							}
				}
			/>
		</PageShell>
	)
}

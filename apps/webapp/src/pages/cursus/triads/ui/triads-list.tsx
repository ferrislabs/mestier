import { Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'
import type { TriadSummary } from '#/pages/cursus/triads/feature/use-triads'
import { TriadsTable } from '#/pages/cursus/triads/ui/triads-table'

export type ClosedFilter = 'all' | 'open' | 'closed'

const STATUS_OPTIONS: SelectOption[] = [
	{ value: 'open', label: 'En cours' },
	{ value: 'closed', label: 'Clôturés' },
]

interface TriadsListProps {
	triads: TriadSummary[]
	promotions: SelectOption[]
	promotionId: string | undefined
	status: ClosedFilter
	isLoading: boolean
	canManage: boolean
	onPromotionChange: (promotionId: string | undefined) => void
	onStatusChange: (status: ClosedFilter) => void
	onOpen: (triadId: string) => void
	onCreate: () => void
}

export function TriadsList({
	triads,
	promotions,
	promotionId,
	status,
	isLoading,
	canManage,
	onPromotionChange,
	onStatusChange,
	onOpen,
	onCreate,
}: TriadsListProps) {
	return (
		<PageShell>
			<PageHeader
				title="Trinômes"
				description="Apprenti, tuteur pédagogique et entreprise d’accueil."
				actions={
					canManage ? (
						<Button type="button" onClick={onCreate}>
							<Plus />
							Nouveau trinôme
						</Button>
					) : null
				}
			/>
			<div className="flex flex-wrap items-center gap-4">
				<div className="w-64">
					<OptionSelect
						id="triads-promotion-filter"
						value={promotionId ?? null}
						options={promotions}
						placeholder="Toutes les promotions"
						noneLabel="Toutes les promotions"
						onChange={(next) => onPromotionChange(next ?? undefined)}
					/>
				</div>
				<div className="w-48">
					<OptionSelect
						id="triads-status-filter"
						value={status === 'all' ? null : status}
						options={STATUS_OPTIONS}
						placeholder="Tous les statuts"
						noneLabel="Tous les statuts"
						onChange={(next) =>
							onStatusChange(
								next === 'open' || next === 'closed' ? next : 'all',
							)
						}
					/>
				</div>
			</div>
			{!isLoading && triads.length === 0 ? (
				<EmptyState title="Aucun trinôme" />
			) : (
				<TriadsTable triads={triads} showPromotion onOpen={onOpen} />
			)}
		</PageShell>
	)
}

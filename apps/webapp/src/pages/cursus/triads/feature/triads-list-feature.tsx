import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
import { usePromotions } from '#/pages/cursus/promotions/feature/use-promotions'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import { TriadCreateFeature } from '#/pages/cursus/triads/feature/triad-create-feature'
import { useTriads } from '#/pages/cursus/triads/feature/use-triads'
import {
	type TriadStatusFilter,
	TriadsList,
} from '#/pages/cursus/triads/ui/triads-list'

export interface TriadFiltersState {
	promotionId: string | undefined
	status: TriadStatusFilter
}

export interface TriadsListFeatureProps extends TriadFiltersState {
	onFiltersChange: (filters: TriadFiltersState) => void
	onOpenTriad: (triadId: string) => void
}

function toClosed(status: TriadStatusFilter): boolean | undefined {
	if (status === 'all') return undefined
	return status === 'closed'
}

export function TriadsListFeature({
	promotionId,
	status,
	onFiltersChange,
	onOpenTriad,
}: TriadsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [creating, setCreating] = useState(false)
	const search = useDebouncedSearch('')
	const triads = useTriads(activeOrganizationId, {
		promotionId,
		closed: toClosed(status),
		q: search.query,
	})
	const promotions = usePromotions(activeOrganizationId, {
		archived: 'active',
		trainingId: undefined,
	})

	return (
		<>
			<TriadsList
				triads={triads.data?.data ?? []}
				total={countOf(triads.data)}
				promotions={(promotions.data?.data ?? []).map((promotion) => ({
					value: promotion.id,
					label: promotion.label,
				}))}
				promotionId={promotionId}
				status={status}
				search={search.text}
				isLoading={triads.isLoading}
				canManage={canManage}
				onSearchChange={search.setText}
				onPromotionChange={(next) =>
					onFiltersChange({ promotionId: next, status })
				}
				onStatusChange={(next) =>
					onFiltersChange({ promotionId, status: next })
				}
				onResetFilters={() => {
					search.reset()
					onFiltersChange({ promotionId: undefined, status: 'open' })
				}}
				onOpen={onOpenTriad}
				onCreate={() => setCreating(true)}
			/>
			{canManage ? (
				<TriadCreateFeature
					open={creating}
					onOpenChange={setCreating}
					onCreated={onOpenTriad}
				/>
			) : null}
		</>
	)
}

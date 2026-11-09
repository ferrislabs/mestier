import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { usePromotions } from '#/pages/cursus/promotions/feature/use-promotions'
import { TriadCreateFeature } from '#/pages/cursus/triads/feature/triad-create-feature'
import { useTriads } from '#/pages/cursus/triads/feature/use-triads'
import {
	type ClosedFilter,
	TriadsList,
} from '#/pages/cursus/triads/ui/triads-list'

export interface TriadsListFeatureProps {
	promotionId: string | undefined
	closed: boolean | undefined
	onFiltersChange: (filters: {
		promotionId: string | undefined
		closed: boolean | undefined
	}) => void
	onOpenTriad: (triadId: string) => void
}

function toStatus(closed: boolean | undefined): ClosedFilter {
	if (closed === undefined) return 'all'
	return closed ? 'closed' : 'open'
}

export function TriadsListFeature({
	promotionId,
	closed,
	onFiltersChange,
	onOpenTriad,
}: TriadsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [creating, setCreating] = useState(false)
	const triads = useTriads(activeOrganizationId, { promotionId, closed })
	const promotions = usePromotions(activeOrganizationId, {
		archived: false,
		trainingId: undefined,
	})

	return (
		<>
			<TriadsList
				triads={triads.data?.data ?? []}
				promotions={(promotions.data?.data ?? []).map((promotion) => ({
					value: promotion.id,
					label: promotion.label,
				}))}
				promotionId={promotionId}
				status={toStatus(closed)}
				isLoading={triads.isLoading}
				canManage={canManage}
				onPromotionChange={(next) =>
					onFiltersChange({ promotionId: next, closed })
				}
				onStatusChange={(next) =>
					onFiltersChange({
						promotionId,
						closed: next === 'all' ? undefined : next === 'closed',
					})
				}
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

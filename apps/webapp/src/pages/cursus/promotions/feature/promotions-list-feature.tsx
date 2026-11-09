import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	type PromotionDialogState,
	PromotionDialogsFeature,
} from '#/pages/cursus/promotions/feature/promotion-dialogs-feature'
import {
	useArchivePromotion,
	usePromotions,
} from '#/pages/cursus/promotions/feature/use-promotions'
import { PromotionsList } from '#/pages/cursus/promotions/ui/promotions-list'
import { useTrainings } from '#/pages/cursus/shared/feature/cursus-queries'

export interface PromotionsListFeatureProps {
	archived: boolean
	trainingId: string | undefined
	onFiltersChange: (filters: {
		archived: boolean
		trainingId: string | undefined
	}) => void
	onOpenPromotion: (promotionId: string) => void
}

export function PromotionsListFeature({
	archived,
	trainingId,
	onFiltersChange,
	onOpenPromotion,
}: PromotionsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [dialog, setDialog] = useState<PromotionDialogState>({ kind: 'none' })
	const promotions = usePromotions(activeOrganizationId, {
		archived,
		trainingId,
	})
	const trainings = useTrainings(activeOrganizationId)
	const archive = useArchivePromotion()

	const trainingRows = trainings.data?.data ?? []
	const trainingNames = Object.fromEntries(
		trainingRows.map((training) => [training.id, training.name]),
	)
	const trainingOptions = trainingRows
		.filter((training) => !training.archived_at)
		.map((training) => ({ value: training.id, label: training.name }))

	return (
		<>
			<PromotionsList
				promotions={promotions.data?.data ?? []}
				trainingNames={trainingNames}
				trainings={trainingOptions}
				trainingId={trainingId}
				archived={archived}
				isLoading={promotions.isLoading}
				canManage={canManage}
				archiveError={mutationErrorMessage(archive.error)}
				onTrainingChange={(next) =>
					onFiltersChange({ archived, trainingId: next })
				}
				onArchivedChange={(next) =>
					onFiltersChange({ archived: next, trainingId })
				}
				onOpen={onOpenPromotion}
				onCreate={() => setDialog({ kind: 'create' })}
				onEdit={(promotion) => setDialog({ kind: 'edit', promotion })}
				onArchive={(promotion) =>
					archive.mutate({
						path: {
							organization_id: activeOrganizationId,
							promotion_id: promotion.id,
						},
					})
				}
			/>
			{canManage ? (
				<PromotionDialogsFeature
					state={dialog}
					trainings={trainingOptions}
					onClose={() => setDialog({ kind: 'none' })}
				/>
			) : null}
		</>
	)
}

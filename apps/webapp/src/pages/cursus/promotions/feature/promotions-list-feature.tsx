import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
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
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import type { ArchiveFilter } from '#/pages/cursus/shared/lib/list-filters'

export interface PromotionFiltersState {
	archived: ArchiveFilter
	trainingId: string | undefined
	q: string | undefined
}

export interface PromotionsListFeatureProps extends PromotionFiltersState {
	onFiltersChange: (filters: PromotionFiltersState) => void
	onOpenPromotion: (promotionId: string) => void
	onManageTrainings?: () => void
}

export function PromotionsListFeature({
	archived,
	trainingId,
	q,
	onFiltersChange,
	onOpenPromotion,
	onManageTrainings,
}: PromotionsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_CATALOG')
	const [dialog, setDialog] = useState<PromotionDialogState>({ kind: 'none' })
	const search = useDebouncedSearch(q ?? '', (next) =>
		onFiltersChange({ archived, trainingId, q: next || undefined }),
	)
	const promotions = usePromotions(activeOrganizationId, {
		archived,
		trainingId,
		q: search.query,
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
				total={countOf(promotions.data)}
				trainingNames={trainingNames}
				trainings={trainingOptions}
				trainingId={trainingId}
				archived={archived}
				search={search.text}
				isLoading={promotions.isLoading}
				canManage={canManage}
				archiveError={mutationErrorMessage(archive.error)}
				onSearchChange={search.setText}
				onTrainingChange={(next) =>
					onFiltersChange({ archived, trainingId: next, q })
				}
				onArchivedChange={(next) =>
					onFiltersChange({ archived: next, trainingId, q })
				}
				onResetFilters={() => {
					search.reset()
					onFiltersChange({
						archived: 'active',
						trainingId: undefined,
						q: undefined,
					})
				}}
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
					onManageTrainings={onManageTrainings}
					state={dialog}
					trainings={trainingOptions}
					onClose={() => setDialog({ kind: 'none' })}
				/>
			) : null}
		</>
	)
}

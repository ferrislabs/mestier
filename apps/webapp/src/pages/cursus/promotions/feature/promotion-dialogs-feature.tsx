import { useActiveOrganization } from '#/hooks/use-active-organization'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	type Promotion,
	useCreatePromotion,
	useUpdatePromotion,
} from '#/pages/cursus/promotions/feature/use-promotions'
import {
	EMPTY_PROMOTION_FORM,
	promotionToForm,
	toPromotionRequest,
} from '#/pages/cursus/promotions/lib/promotion-form'
import { PromotionFormDialog } from '#/pages/cursus/promotions/ui/promotion-form-dialog'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'

export type PromotionDialogState =
	| { kind: 'none' }
	| { kind: 'create' }
	| { kind: 'edit'; promotion: Promotion }

interface PromotionDialogsFeatureProps {
	state: PromotionDialogState
	trainings: SelectOption[]
	onClose: () => void
	onCreated?: (promotionId: string) => void
}

export function PromotionDialogsFeature({
	state,
	trainings,
	onClose,
	onCreated,
}: PromotionDialogsFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const create = useCreatePromotion()
	const update = useUpdatePromotion()
	const editing = state.kind === 'edit'

	return (
		<PromotionFormDialog
			open={state.kind !== 'none'}
			title={editing ? 'Modifier la promotion' : 'Nouvelle promotion'}
			submitLabel={editing ? 'Enregistrer' : 'Créer la promotion'}
			initial={
				state.kind === 'edit'
					? promotionToForm(state.promotion)
					: EMPTY_PROMOTION_FORM
			}
			trainings={trainings}
			isPending={create.isPending || update.isPending}
			error={mutationErrorMessage(editing ? update.error : create.error)}
			onOpenChange={(next) => {
				if (!next) {
					create.reset()
					update.reset()
					onClose()
				}
			}}
			onSubmit={(values) => {
				if (state.kind === 'edit') {
					update.mutate(
						{
							path: {
								organization_id: activeOrganizationId,
								promotion_id: state.promotion.id,
							},
							body: toPromotionRequest(values),
						},
						{ onSuccess: onClose },
					)
					return
				}
				create.mutate(
					{
						path: { organization_id: activeOrganizationId },
						body: toPromotionRequest(values),
					},
					{
						onSuccess: (response) => {
							onClose()
							onCreated?.(response.data.id)
						},
					},
				)
			}}
		/>
	)
}

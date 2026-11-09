import { useActiveOrganization } from '#/hooks/use-active-organization'
import { mutationErrorMessage } from '#/lib/api-error'
import { usePromotions } from '#/pages/cursus/promotions/feature/use-promotions'
import {
	useOrganizationMembers,
	useParticipantRecords,
} from '#/pages/cursus/shared/feature/cursus-queries'
import {
	apprenticeOptions,
	type SelectOption,
} from '#/pages/cursus/shared/lib/options'
import { useCreateTriad } from '#/pages/cursus/triads/feature/use-triads'
import { toTriadRequest } from '#/pages/cursus/triads/lib/triad-form'
import { TriadFormDialog } from '#/pages/cursus/triads/ui/triad-form-dialog'

interface TriadCreateFeatureProps {
	open: boolean
	fixedPromotionId?: string
	onOpenChange: (open: boolean) => void
	onCreated: (triadId: string) => void
}

export function TriadCreateFeature({
	open,
	fixedPromotionId,
	onOpenChange,
	onCreated,
}: TriadCreateFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const apprentices = useParticipantRecords(
		activeOrganizationId,
		{ kind: 'apprentice' },
		open,
	)
	const members = useOrganizationMembers(activeOrganizationId, open)
	const promotions = usePromotions(
		activeOrganizationId,
		{ archived: false, trainingId: undefined },
		open && !fixedPromotionId,
	)
	const createTriad = useCreateTriad()

	const memberOptions: SelectOption[] = members.map((member) => ({
		value: member.id,
		label: member.display_name,
	}))
	const promotionOptions: SelectOption[] = (promotions.data?.data ?? []).map(
		(promotion) => ({ value: promotion.id, label: promotion.label }),
	)

	return (
		<TriadFormDialog
			open={open}
			fixedPromotionId={fixedPromotionId}
			promotions={promotionOptions}
			apprentices={apprenticeOptions(apprentices.items)}
			members={memberOptions}
			isLoadingOptions={apprentices.isLoading}
			isPending={createTriad.isPending}
			error={mutationErrorMessage(createTriad.error)}
			onOpenChange={(next) => {
				if (!next) createTriad.reset()
				onOpenChange(next)
			}}
			onSubmit={(values) =>
				createTriad.mutate(
					{
						path: { organization_id: activeOrganizationId },
						body: toTriadRequest(values),
					},
					{
						onSuccess: (response) => {
							onOpenChange(false)
							onCreated(response.data.id)
						},
					},
				)
			}
		/>
	)
}

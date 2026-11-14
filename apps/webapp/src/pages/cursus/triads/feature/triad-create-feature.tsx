import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePromotions } from '#/pages/cursus/promotions/feature/use-promotions'
import { useSchoolTutors } from '#/pages/cursus/school-tutors/feature/use-school-tutors'
import {
	schoolTutorOptions,
	triadTutorErrors,
} from '#/pages/cursus/school-tutors/lib/school-tutors'
import { useParticipantRecords } from '#/pages/cursus/shared/feature/cursus-queries'
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
	onDeclareSchoolTutor?: () => void
}

export function TriadCreateFeature({
	open,
	fixedPromotionId,
	onOpenChange,
	onCreated,
	onDeclareSchoolTutor,
}: TriadCreateFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const apprentices = useParticipantRecords(
		activeOrganizationId,
		{ kind: 'apprentice' },
		open,
	)
	const schoolTutors = useSchoolTutors(
		activeOrganizationId,
		{ archived: 'active', page: 1, per_page: 100 },
		open,
	)
	const promotions = usePromotions(
		activeOrganizationId,
		{ archived: 'active', trainingId: undefined },
		open && !fixedPromotionId,
	)
	const createTriad = useCreateTriad()

	const errors = triadTutorErrors(createTriad.error)
	const promotionOptions: SelectOption[] = (promotions.data?.data ?? []).map(
		(promotion) => ({ value: promotion.id, label: promotion.label }),
	)

	return (
		<TriadFormDialog
			open={open}
			fixedPromotionId={fixedPromotionId}
			promotions={promotionOptions}
			apprentices={apprenticeOptions(apprentices.items)}
			schoolTutors={schoolTutorOptions(schoolTutors.data?.data ?? [], null)}
			isLoadingOptions={apprentices.isLoading || schoolTutors.isLoading}
			isPending={createTriad.isPending}
			error={errors.general}
			tutorError={errors.field}
			onDeclareSchoolTutor={onDeclareSchoolTutor}
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

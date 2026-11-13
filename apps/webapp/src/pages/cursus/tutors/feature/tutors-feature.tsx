import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useHostCompanies } from '#/pages/cursus/host-companies/feature/use-host-companies'
import {
	type ParticipantsFilters,
	ParticipantsListFeature,
} from '#/pages/cursus/participants/feature/participants-list-feature'
import { useCreateParticipant } from '#/pages/cursus/participants/feature/use-participants'
import { toCreateParticipantRequest } from '#/pages/cursus/participants/lib/participants'
import { SchoolTutorsListFeature } from '#/pages/cursus/school-tutors/feature/school-tutors-list-feature'
import {
	useCreateSchoolTutor,
	useSchoolTutors,
} from '#/pages/cursus/school-tutors/feature/use-school-tutors'
import {
	availableMembers,
	createSchoolTutorError,
	toCreateSchoolTutorRequest,
} from '#/pages/cursus/school-tutors/lib/school-tutors'
import { useOrganizationMembers } from '#/pages/cursus/shared/feature/cursus-queries'
import { useTutorCounts } from '#/pages/cursus/tutors/feature/use-tutor-counts'
import type { TutorTab } from '#/pages/cursus/tutors/lib/tutors'
import { NewTutorDialog } from '#/pages/cursus/tutors/ui/new-tutor-dialog'
import { TutorsHeader } from '#/pages/cursus/tutors/ui/tutors-header'

export interface TutorsFeatureProps extends ParticipantsFilters {
	tab: TutorTab
	onTabChange: (tab: TutorTab) => void
	onFiltersChange: (filters: ParticipantsFilters) => void
	onOpenParticipant: (participantId: string) => void
}

export function TutorsFeature({
	tab,
	status,
	hostCompanyId,
	onTabChange,
	onFiltersChange,
	onOpenParticipant,
}: TutorsFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_PARTICIPANTS')
	const [createOpen, setCreateOpen] = useState(false)
	const counts = useTutorCounts(activeOrganizationId)
	const members = useOrganizationMembers(activeOrganizationId, createOpen)
	const declared = useSchoolTutors(
		activeOrganizationId,
		{ archived: 'all', page: 1, per_page: 100 },
		createOpen,
	)
	const hostCompanies = useHostCompanies(activeOrganizationId, {
		per_page: 100,
	})
	const createParticipant = useCreateParticipant()
	const createSchoolTutor = useCreateSchoolTutor()

	const openCreate = () => {
		createParticipant.reset()
		createSchoolTutor.reset()
		setCreateOpen(true)
	}
	const header = (
		<TutorsHeader
			tab={tab}
			companyCount={counts.company}
			schoolCount={counts.school}
			onTabChange={onTabChange}
			canManage={canManage}
			onCreate={openCreate}
		/>
	)

	return (
		<>
			{tab === 'ecole' ? (
				<SchoolTutorsListFeature header={header} canManage={canManage} />
			) : (
				<ParticipantsListFeature
					kind="company_tutor"
					status={status}
					hostCompanyId={hostCompanyId}
					onFiltersChange={onFiltersChange}
					onOpenParticipant={onOpenParticipant}
					header={header}
					onCreate={openCreate}
				/>
			)}
			{canManage ? (
				<NewTutorDialog
					key={String(createOpen)}
					open={createOpen}
					hostCompanies={(hostCompanies.data?.data ?? []).map((company) => ({
						id: company.id,
						name: company.name,
					}))}
					members={availableMembers(members, declared.data?.data ?? [])}
					isLoadingMembers={declared.isLoading}
					company={{
						isPending: createParticipant.isPending,
						error: mutationErrorMessage(createParticipant.error),
						onSubmit: (values) =>
							createParticipant.mutate(
								{
									path: { organization_id: activeOrganizationId },
									body: toCreateParticipantRequest('company_tutor', values),
								},
								{
									onSuccess: (response) => {
										setCreateOpen(false)
										onOpenParticipant(response.data.id)
									},
								},
							),
					}}
					school={{
						isPending: createSchoolTutor.isPending,
						error: createSchoolTutorError(createSchoolTutor.error),
						onSubmit: (values) =>
							createSchoolTutor.mutate(
								{
									path: { organization_id: activeOrganizationId },
									body: toCreateSchoolTutorRequest(values),
								},
								{
									onSuccess: () => {
										setCreateOpen(false)
										onTabChange('ecole')
									},
								},
							),
					}}
					onOpenChange={setCreateOpen}
				/>
			) : null}
		</>
	)
}

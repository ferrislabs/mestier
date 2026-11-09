import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useArchiveHostCompany,
	useHostCompany,
	useUpdateHostCompany,
} from '#/pages/cursus/host-companies/feature/use-host-companies'
import { toHostCompanyRequest } from '#/pages/cursus/host-companies/lib/host-companies'
import {
	HostCompanyDetailUI,
	type HostCompanyTutorVM,
} from '#/pages/cursus/host-companies/ui/host-company-detail-ui'
import {
	useParticipantDetails,
	useParticipants,
} from '#/pages/cursus/participants/feature/use-participants'
import { participantName } from '#/pages/cursus/participants/lib/participants'

export interface HostCompanyDetailFeatureProps {
	hostCompanyId: string
	onOpenParticipant: (participantId: string) => void
}

export function HostCompanyDetailFeature({
	hostCompanyId,
	onOpenParticipant,
}: HostCompanyDetailFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const query = useHostCompany(activeOrganizationId, hostCompanyId)
	const updateCompany = useUpdateHostCompany()
	const archiveCompany = useArchiveHostCompany()
	const tutorList = useParticipants(activeOrganizationId, {
		kind: 'company_tutor',
		per_page: 100,
	})
	const tutorParticipants = tutorList.data?.data ?? []
	const tutorDetails = useParticipantDetails(
		activeOrganizationId,
		tutorParticipants.map((participant) => participant.id),
	)

	const tutors: HostCompanyTutorVM[] = tutorParticipants.flatMap(
		(participant, index) => {
			const record = tutorDetails[index]?.data?.data.company_tutors.find(
				(tutor) => tutor.host_company_id === hostCompanyId,
			)
			return record
				? [
						{
							participantId: participant.id,
							name: participantName(participant),
							jobTitle: record.job_title ?? null,
						},
					]
				: []
		},
	)

	const path = {
		organization_id: activeOrganizationId,
		host_company_id: hostCompanyId,
	}

	return (
		<HostCompanyDetailUI
			company={query.data?.data ?? null}
			isLoading={query.isLoading}
			loadError={mutationErrorMessage(query.error)}
			canManage={canManage}
			isSaving={updateCompany.isPending}
			saveError={mutationErrorMessage(updateCompany.error)}
			isArchiving={archiveCompany.isPending}
			archiveError={mutationErrorMessage(archiveCompany.error)}
			tutors={tutors}
			onSave={(values) =>
				updateCompany.mutate({ path, body: toHostCompanyRequest(values) })
			}
			onArchive={() => archiveCompany.mutate({ path })}
			onOpenParticipant={onOpenParticipant}
		/>
	)
}

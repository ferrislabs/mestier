import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useHostCompanies } from '#/pages/cursus/host-companies/feature/use-host-companies'
import {
	useAddCompanyTutor,
	useDeactivateParticipant,
	useInviteParticipant,
	useParticipant,
	useReactivateParticipant,
	useRemoveCompanyTutor,
	useUpdateParticipant,
	useUpsertApprentice,
} from '#/pages/cursus/participants/feature/use-participants'
import { conflictMessage } from '#/pages/cursus/participants/lib/forms'
import {
	buildInvitationLink,
	participantName,
	toApprenticeRequest,
	toUpdateParticipantRequest,
} from '#/pages/cursus/participants/lib/participants'
import { InvitationLinkDialog } from '#/pages/cursus/participants/ui/invitation-link-dialog'
import { ParticipantDetailUI } from '#/pages/cursus/participants/ui/participant-detail-ui'

const TUTOR_LINKED_MESSAGE =
	'Cette fiche tuteur est liée à un contrat : retirez-la du contrat avant de la supprimer.'

export interface ParticipantDetailFeatureProps {
	participantId: string
}

export function ParticipantDetailFeature({
	participantId,
}: ParticipantDetailFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_PARTICIPANTS')
	const query = useParticipant(activeOrganizationId, participantId)
	const hostCompanies = useHostCompanies(activeOrganizationId, {
		per_page: 100,
	})
	const updateParticipant = useUpdateParticipant()
	const upsertApprentice = useUpsertApprentice()
	const addTutor = useAddCompanyTutor()
	const removeTutor = useRemoveCompanyTutor()
	const invite = useInviteParticipant()
	const deactivate = useDeactivateParticipant()
	const reactivate = useReactivateParticipant()
	const [invitation, setInvitation] = useState<{
		link: string
		expiresAt: string
	} | null>(null)

	const participant = query.data?.data ?? null
	const path = {
		organization_id: activeOrganizationId,
		participant_id: participantId,
	}
	const isBusy =
		updateParticipant.isPending ||
		upsertApprentice.isPending ||
		addTutor.isPending ||
		removeTutor.isPending ||
		invite.isPending ||
		deactivate.isPending ||
		reactivate.isPending

	return (
		<>
			<ParticipantDetailUI
				participant={participant}
				isLoading={query.isLoading}
				loadError={mutationErrorMessage(query.error)}
				canManage={canManage}
				isBusy={isBusy}
				hostCompanies={(hostCompanies.data?.data ?? []).map((company) => ({
					id: company.id,
					name: company.name,
				}))}
				errors={{
					identity: mutationErrorMessage(updateParticipant.error),
					apprentice: mutationErrorMessage(upsertApprentice.error),
					tutors:
						conflictMessage(removeTutor.error, TUTOR_LINKED_MESSAGE) ??
						mutationErrorMessage(removeTutor.error) ??
						mutationErrorMessage(addTutor.error),
					status:
						mutationErrorMessage(invite.error) ??
						mutationErrorMessage(deactivate.error) ??
						mutationErrorMessage(reactivate.error),
				}}
				onSaveIdentity={(values) =>
					updateParticipant.mutate({
						path,
						body: toUpdateParticipantRequest(values),
					})
				}
				onSaveApprentice={(values) =>
					upsertApprentice.mutate({ path, body: toApprenticeRequest(values) })
				}
				onAddTutor={(values) =>
					addTutor.mutate({
						path,
						body: {
							host_company_id: values.hostCompanyId,
							job_title: values.jobTitle.trim() || null,
						},
					})
				}
				onRemoveTutor={(companyTutorId) =>
					removeTutor.mutate({
						path: {
							organization_id: activeOrganizationId,
							company_tutor_id: companyTutorId,
						},
					})
				}
				onInvite={() =>
					invite.mutate(
						{ path },
						{
							onSuccess: (response) =>
								setInvitation({
									link: buildInvitationLink(response.data.token),
									expiresAt: response.data.expires_at,
								}),
						},
					)
				}
				onDeactivate={() => deactivate.mutate({ path })}
				onReactivate={() => reactivate.mutate({ path })}
			/>
			<InvitationLinkDialog
				participantName={participant ? participantName(participant) : ''}
				link={invitation?.link ?? null}
				expiresAt={invitation?.expiresAt ?? null}
				onClose={() => setInvitation(null)}
			/>
		</>
	)
}

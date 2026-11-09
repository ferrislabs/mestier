import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useHostCompanies } from '#/pages/cursus/host-companies/feature/use-host-companies'
import {
	PARTICIPANTS_PER_PAGE,
	useCreateParticipant,
	useParticipantDetails,
	useParticipants,
} from '#/pages/cursus/participants/feature/use-participants'
import {
	kindInfo,
	type ParticipantKind,
	type ParticipantStatus,
	participantName,
	toCreateParticipantRequest,
} from '#/pages/cursus/participants/lib/participants'
import { CreateParticipantDialog } from '#/pages/cursus/participants/ui/create-participant-dialog'
import {
	type ParticipantRowVM,
	ParticipantsListUI,
} from '#/pages/cursus/participants/ui/participants-list-ui'

export interface ParticipantsListFeatureProps {
	kind: ParticipantKind | undefined
	status: ParticipantStatus | undefined
	onFiltersChange: (filters: {
		kind: ParticipantKind | undefined
		status: ParticipantStatus | undefined
	}) => void
	onOpenParticipant: (participantId: string) => void
}

export function ParticipantsListFeature({
	kind,
	status,
	onFiltersChange,
	onOpenParticipant,
}: ParticipantsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_PARTICIPANTS')
	const [page, setPage] = useState(1)
	const [createOpen, setCreateOpen] = useState(false)

	const list = useParticipants(activeOrganizationId, {
		kind,
		status,
		page,
		per_page: PARTICIPANTS_PER_PAGE,
	})
	const participants = list.data?.data ?? []
	const details = useParticipantDetails(
		activeOrganizationId,
		participants.map((participant) => participant.id),
	)
	const hostCompanies = useHostCompanies(activeOrganizationId, {
		per_page: 100,
	})
	const createParticipant = useCreateParticipant()

	const rows: ParticipantRowVM[] = participants.map((participant, index) => {
		const info = kindInfo(details[index]?.data?.data)
		return {
			id: participant.id,
			name: participantName(participant),
			email: participant.email,
			kinds: info?.kinds ?? null,
			hostCompanyNames: info?.hostCompanyNames ?? null,
			status: participant.status,
		}
	})

	const total = list.data?.pagination?.total ?? participants.length
	const pageCount = Math.max(1, Math.ceil(total / PARTICIPANTS_PER_PAGE))

	return (
		<>
			<ParticipantsListUI
				rows={rows}
				isLoading={list.isLoading}
				error={mutationErrorMessage(list.error)}
				kind={kind}
				status={status}
				canManage={canManage}
				page={page}
				pageCount={pageCount}
				onKindChange={(next) => {
					setPage(1)
					onFiltersChange({ kind: next, status })
				}}
				onStatusChange={(next) => {
					setPage(1)
					onFiltersChange({ kind, status: next })
				}}
				onPageChange={setPage}
				onCreate={() => setCreateOpen(true)}
				onOpen={onOpenParticipant}
			/>
			{canManage ? (
				<CreateParticipantDialog
					key={String(createOpen)}
					open={createOpen}
					hostCompanies={(hostCompanies.data?.data ?? []).map((company) => ({
						id: company.id,
						name: company.name,
					}))}
					isPending={createParticipant.isPending}
					error={mutationErrorMessage(createParticipant.error)}
					onOpenChange={setCreateOpen}
					onSubmit={(values) =>
						createParticipant.mutate(
							{
								path: { organization_id: activeOrganizationId },
								body: toCreateParticipantRequest(values),
							},
							{
								onSuccess: (response) => {
									setCreateOpen(false)
									onOpenParticipant(response.data.id)
								},
							},
						)
					}
				/>
			) : null}
		</>
	)
}

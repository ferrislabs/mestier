import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { useHostCompanies } from '#/pages/cursus/host-companies/feature/use-host-companies'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
import {
	PARTICIPANTS_PER_PAGE,
	useCreateParticipant,
	useParticipants,
} from '#/pages/cursus/participants/feature/use-participants'
import {
	type ParticipantKind,
	type ParticipantStatus,
	participantName,
	toCreateParticipantRequest,
	tutorInfo,
} from '#/pages/cursus/participants/lib/participants'
import { CreateParticipantDialog } from '#/pages/cursus/participants/ui/create-participant-dialog'
import {
	type ParticipantRowVM,
	ParticipantsListUI,
} from '#/pages/cursus/participants/ui/participants-list-ui'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import { searchQuery } from '#/pages/cursus/shared/lib/list-filters'

export interface ParticipantsFilters {
	status: ParticipantStatus | undefined
	hostCompanyId: string | undefined
}

export interface ParticipantsListFeatureProps extends ParticipantsFilters {
	kind: ParticipantKind
	onFiltersChange: (filters: ParticipantsFilters) => void
	onOpenParticipant: (participantId: string) => void
}

export function ParticipantsListFeature({
	kind,
	status,
	hostCompanyId,
	onFiltersChange,
	onOpenParticipant,
}: ParticipantsListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_PARTICIPANTS')
	const [page, setPage] = useState(1)
	const [createOpen, setCreateOpen] = useState(false)
	const search = useDebouncedSearch('', () => setPage(1))
	const companyFilter = kind === 'company_tutor' ? hostCompanyId : undefined

	const list = useParticipants(activeOrganizationId, {
		kind,
		status,
		...(companyFilter ? { host_company_id: companyFilter } : {}),
		...searchQuery(search.query),
		page,
		per_page: PARTICIPANTS_PER_PAGE,
	})
	const hostCompanies = useHostCompanies(activeOrganizationId, {
		per_page: 100,
	})
	const createParticipant = useCreateParticipant()
	const companyOptions = (hostCompanies.data?.data ?? []).map((company) => ({
		id: company.id,
		name: company.name,
	}))

	const rows: ParticipantRowVM[] = (list.data?.data ?? []).map(
		(participant) => ({
			id: participant.id,
			name: participantName(participant),
			email: participant.email,
			...tutorInfo(participant, companyFilter),
			status: participant.status,
		}),
	)

	const total = countOf(list.data)
	const pageCount = Math.max(1, Math.ceil((total ?? 0) / PARTICIPANTS_PER_PAGE))

	function changeFilters(next: ParticipantsFilters) {
		setPage(1)
		onFiltersChange(next)
	}

	return (
		<>
			<ParticipantsListUI
				kind={kind}
				rows={rows}
				total={total}
				search={search.text}
				isLoading={list.isLoading}
				error={mutationErrorMessage(list.error)}
				status={status}
				hostCompanyId={companyFilter}
				hostCompanies={companyOptions}
				canManage={canManage}
				page={page}
				pageCount={pageCount}
				onSearchChange={search.setText}
				onResetFilters={() => {
					search.reset()
					changeFilters({ status: undefined, hostCompanyId: undefined })
				}}
				onStatusChange={(next) =>
					changeFilters({ status: next, hostCompanyId: companyFilter })
				}
				onHostCompanyChange={(next) =>
					changeFilters({ status, hostCompanyId: next })
				}
				onPageChange={setPage}
				onCreate={() => setCreateOpen(true)}
				onOpen={onOpenParticipant}
			/>
			{canManage ? (
				<CreateParticipantDialog
					key={String(createOpen)}
					kind={kind}
					open={createOpen}
					hostCompanies={companyOptions}
					isPending={createParticipant.isPending}
					error={mutationErrorMessage(createParticipant.error)}
					onOpenChange={setCreateOpen}
					onSubmit={(values) =>
						createParticipant.mutate(
							{
								path: { organization_id: activeOrganizationId },
								body: toCreateParticipantRequest(kind, values),
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

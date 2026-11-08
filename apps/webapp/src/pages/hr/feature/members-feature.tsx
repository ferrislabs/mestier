import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateInvitation,
	usePendingInvitations,
	useRevokeInvitation,
} from '#/hooks/use-invitations'
import {
	useCreateMember,
	useDeleteMember,
	useReferenceCatalog,
	useRemoveEmployeeProfile,
	useUpdateMember,
	useUpsertEmployeeProfile,
} from '#/hooks/use-reference-catalog'
import { isForbiddenError, mutationErrorMessage } from '#/lib/api-error'
import { MemberRoleCell } from '#/pages/hr/feature/member-role-cell'
import { RhScreen } from '#/pages/hr/feature/rh-screen'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import {
	type ContractFilter,
	filterMembers,
	type MemberFilters,
	type StatusFilter,
} from '#/pages/hr/lib/rh/members-filter'
import type { Member } from '#/pages/hr/lib/rh/types'
import { accessState, type MemberFormValues } from '#/pages/hr/types'
import { InviteMemberSheet } from '#/pages/hr/ui/invite-member-sheet'
import type { MemberRealInfo } from '#/pages/hr/ui/rh/members/member-access'
import { MemberCostDialog } from '#/pages/hr/ui/rh/members/member-cost-dialog'
import { MembersPage } from '#/pages/hr/ui/rh/members/members-page'
import type { PendingInvitationRow } from '#/pages/hr/ui/rh/members/pending-invitations'
import { useRhState } from '#/pages/hr/ui/rh/shared/rh-context'

export interface MembersFeatureProps {
	status: StatusFilter
	contract: ContractFilter
	onFilterChange: (patch: {
		status: StatusFilter
		contract: ContractFilter
	}) => void
	onMemberCreated: (memberId: string) => void
}

export function MembersFeature(props: MembersFeatureProps) {
	return (
		<RhScreen loadingLabel="Chargement de l’équipe…">
			<MembersScreen {...props} />
		</RhScreen>
	)
}

const EMPTY_VALUES: MemberFormValues = {
	lastName: '',
	firstName: '',
	hourlyRate: '',
	isSalaried: false,
	monthlyCost: '',
}

function MembersScreen({
	status,
	contract,
	onFilterChange,
	onMemberCreated,
}: MembersFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const state = useRhState()
	const today = todayIso()

	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const createMember = useCreateMember(organizationId)
	const updateMember = useUpdateMember()
	const deleteMember = useDeleteMember()
	const upsertProfile = useUpsertEmployeeProfile()
	const removeProfile = useRemoveEmployeeProfile()
	const pendingInvitations = usePendingInvitations(organizationId)
	const createInvitation = useCreateInvitation(organizationId)
	const revokeInvitation = useRevokeInvitation()

	const [query, setQuery] = useState('')
	const [createOpen, setCreateOpen] = useState(false)
	const [editing, setEditing] = useState<string | null>(null)
	const [editValues, setEditValues] = useState<MemberFormValues>(EMPTY_VALUES)
	const [isSaving, setIsSaving] = useState(false)
	const [inviteTarget, setInviteTarget] = useState<{
		id: string
		name: string
	} | null>(null)
	const [inviteToken, setInviteToken] = useState<string | null>(null)
	const [revokingInvitationId, setRevokingInvitationId] = useState<
		string | null
	>(null)

	const filters: MemberFilters = { query, status, contract }
	const setFilters = (next: MemberFilters) => {
		setQuery(next.query)
		if (next.status !== status || next.contract !== contract)
			onFilterChange({ status: next.status, contract: next.contract })
	}

	const createForm = useForm({
		defaultValues: EMPTY_VALUES,
		onSubmit: async ({ value }) => {
			const created = await createMember.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					last_name: value.lastName.trim(),
					first_name: value.firstName.trim() || null,
				},
			})
			const rateCents = eurosToCents(value.hourlyRate)
			if (rateCents !== null || value.isSalaried) {
				await upsertProfile.mutateAsync({
					path: { member_id: created.data.id },
					body: {
						hourly_rate_cents: rateCents,
						is_salaried: value.isSalaried,
						monthly_cost_cents: value.isSalaried
							? eurosToCents(value.monthlyCost)
							: null,
						weekly_contract_minutes: 0,
					},
				})
			}
			createForm.reset()
			setCreateOpen(false)
			onMemberCreated(created.data.id)
		},
	})

	const realMembers = catalog.members.data?.data ?? []
	const profileByMember = new Map(
		(catalog.employeeProfiles.data?.data ?? []).map((profile) => [
			profile.member_id,
			profile,
		]),
	)
	const invitations = pendingInvitations.data?.data ?? []
	const pendingByMemberId = new Set(
		invitations
			.map((invitation) => invitation.member_id)
			.filter((id): id is string => Boolean(id)),
	)

	const realInfo = new Map<string, MemberRealInfo>(
		realMembers.map((member) => {
			const profile = profileByMember.get(member.id)
			return [
				member.id,
				{
					access: accessState(member, pendingByMemberId.has(member.id)),
					hourlyRateCents: profile?.hourly_rate_cents ?? null,
					isSalaried: profile?.is_salaried ?? false,
					monthlyCostCents: profile?.monthly_cost_cents ?? null,
					effectiveHourlyRateCents:
						profile?.effective_hourly_rate_cents ?? null,
					weeklyContractMinutes: profile?.weekly_contract_minutes ?? null,
				},
			]
		}),
	)

	const nameOf = (memberId: string) => {
		const member = state.members.find((item) => item.id === memberId)
		return member ? fullName(member) : 'Personne inconnue'
	}
	const pendingRows: PendingInvitationRow[] = invitations.map((invitation) => ({
		id: invitation.id,
		memberName: invitation.member_id
			? nameOf(invitation.member_id)
			: 'Personne inconnue',
		expiresAt: invitation.expires_at,
	}))

	const hrDataRestricted = isForbiddenError(catalog.employeeProfiles.error)

	const error =
		(hrDataRestricted ? null : catalog.employeeProfiles.error) ??
		createMember.error ??
		updateMember.error ??
		deleteMember.error ??
		upsertProfile.error ??
		removeProfile.error ??
		revokeInvitation.error

	const handleGenerateInvite = async () => {
		if (!inviteTarget) return
		const created = await createInvitation.mutateAsync({
			path: { organization_id: organizationId },
			body: { member_id: inviteTarget.id },
		})
		setInviteToken(created.data.token)
	}

	const handleRevokeInvitation = async (invitationId: string) => {
		setRevokingInvitationId(invitationId)
		try {
			await revokeInvitation.mutateAsync({
				path: { invitation_id: invitationId },
			})
		} finally {
			setRevokingInvitationId(null)
		}
	}

	const handleEdit = (member: Member) => {
		const real = realMembers.find((item) => item.id === member.id)
		const info = realInfo.get(member.id)
		if (!real) return
		setEditValues({
			lastName: real.last_name,
			firstName: real.first_name ?? '',
			hourlyRate: centsToEuros(info?.hourlyRateCents ?? null),
			isSalaried: info?.isSalaried ?? false,
			monthlyCost: centsToEuros(info?.monthlyCostCents ?? null),
		})
		setEditing(member.id)
	}

	const handleSaveEdit = async () => {
		if (!editing) return
		setIsSaving(true)
		try {
			await updateMember.mutateAsync({
				path: { member_id: editing },
				body: {
					last_name: editValues.lastName.trim(),
					first_name: editValues.firstName.trim() || null,
				},
			})
			const rateCents = eurosToCents(editValues.hourlyRate)
			const existingProfile = profileByMember.get(editing)
			if (rateCents !== null || editValues.isSalaried) {
				await upsertProfile.mutateAsync({
					path: { member_id: editing },
					body: {
						hourly_rate_cents: rateCents,
						is_salaried: editValues.isSalaried,
						monthly_cost_cents: editValues.isSalaried
							? eurosToCents(editValues.monthlyCost)
							: null,
						weekly_contract_minutes:
							existingProfile?.weekly_contract_minutes ?? 0,
					},
				})
			} else if (existingProfile) {
				await removeProfile.mutateAsync({ path: { member_id: editing } })
			}
			setEditing(null)
		} finally {
			setIsSaving(false)
		}
	}

	const handleCreateOpenChange = (open: boolean) => {
		setCreateOpen(open)
		if (!open) createForm.reset()
	}

	return (
		<createForm.Subscribe selector={(formState) => formState.values}>
			{(createValues) => (
				<MembersPage
					state={state}
					members={filterMembers(state.members, filters, today)}
					today={today}
					filters={filters}
					onFiltersChange={setFilters}
					error={mutationErrorMessage(error)}
					hrDataRestricted={hrDataRestricted}
					realInfo={realInfo}
					pendingInvitations={pendingRows}
					revokingInvitationId={revokingInvitationId}
					renderRoles={(memberId) => <MemberRoleCell memberId={memberId} />}
					onCreate={() => setCreateOpen(true)}
					onEdit={handleEdit}
					onInvite={(member) => {
						setInviteTarget({ id: member.id, name: fullName(member) })
						setInviteToken(null)
					}}
					onDelete={(member) =>
						void deleteMember.mutateAsync({ path: { member_id: member.id } })
					}
					onRevokeInvitation={(invitationId) =>
						void handleRevokeInvitation(invitationId)
					}
					dialogs={
						<>
							<MemberCostDialog
								idPrefix="create-member"
								open={createOpen}
								title="Nouveau membre"
								description="Le profil RH est optionnel : laissez le taux horaire vide pour un siège sans profil, renseignez-le pour lui en attacher un — ou marquez la personne comme salariée si elle n'a pas de coût horaire."
								submitLabel="Ajouter"
								values={createValues}
								isPending={createMember.isPending || upsertProfile.isPending}
								onChange={(patch) => {
									for (const key of Object.keys(
										patch,
									) as (keyof MemberFormValues)[]) {
										createForm.setFieldValue(key, patch[key] ?? '')
									}
								}}
								onSubmit={() => void createForm.handleSubmit()}
								onOpenChange={handleCreateOpenChange}
							/>
							<MemberCostDialog
								idPrefix="edit-member"
								open={editing !== null}
								title="Modifier le membre"
								description="Nom, prénom et coût du membre. Ses coordonnées, contrats et semaine type se gèrent depuis sa fiche."
								submitLabel="Enregistrer"
								values={editValues}
								isPending={isSaving}
								onChange={(patch) =>
									setEditValues((current) => ({ ...current, ...patch }))
								}
								onSubmit={() => void handleSaveEdit()}
								onOpenChange={(open) => {
									if (!open) setEditing(null)
								}}
							/>
							<InviteMemberSheet
								open={inviteTarget !== null}
								memberName={inviteTarget?.name ?? ''}
								token={inviteToken}
								isGenerating={createInvitation.isPending}
								error={mutationErrorMessage(createInvitation.error)}
								onOpenChange={(open) => {
									if (!open) {
										setInviteTarget(null)
										setInviteToken(null)
									}
								}}
								onGenerate={() => void handleGenerateInvite()}
							/>
						</>
					}
				/>
			)}
		</createForm.Subscribe>
	)
}

function eurosToCents(value: string): number | null {
	const normalized = value.replace(',', '.').trim()
	if (normalized === '') return null
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) return null
	return Math.round(parsed * 100)
}

function centsToEuros(value: number | null): string {
	if (value === null) return ''
	return (value / 100).toFixed(2).replace('.', ',')
}

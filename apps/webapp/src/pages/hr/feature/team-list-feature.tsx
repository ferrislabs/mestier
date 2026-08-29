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
import { accessState, type MemberFormValues } from '#/pages/hr/types'
import { InviteMemberSheet } from '#/pages/hr/ui/invite-member-sheet'
import {
	type MemberDraft,
	type PendingInvitationRow,
	TeamListUI,
	type TeamMemberRow,
} from '#/pages/hr/ui/team-list-ui'

export function TeamListFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<TeamDirectory
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

interface TeamDirectoryProps {
	organizationId: string
	organizationName: string
	organizationSlug: string
}

function TeamDirectory({
	organizationId,
	organizationName,
	organizationSlug,
}: TeamDirectoryProps) {
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

	const [search, setSearch] = useState('')
	const [createMemberDialogOpen, setCreateMemberDialogOpen] = useState(false)
	const [draft, setDraft] = useState<MemberDraft | null>(null)
	const [isSaving, setIsSaving] = useState(false)
	const [inviteTarget, setInviteTarget] = useState<{
		id: string
		name: string
	} | null>(null)
	const [inviteToken, setInviteToken] = useState<string | null>(null)
	const [revokingInvitationId, setRevokingInvitationId] = useState<
		string | null
	>(null)

	const memberForm = useForm({
		defaultValues: {
			lastName: '',
			firstName: '',
			hourlyRate: '',
			isSalaried: false,
			monthlyCost: '',
		} satisfies MemberFormValues,
		onSubmit: async ({ value }) => {
			const created = await createMember.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					last_name: value.lastName.trim(),
					first_name: value.firstName.trim() || null,
				},
			})
			const rateCents = eurosToCents(value.hourlyRate)
			const monthlyCents = eurosToCents(value.monthlyCost)
			if (rateCents !== null || value.isSalaried) {
				await upsertProfile.mutateAsync({
					path: { member_id: created.data.id },
					body: {
						hourly_rate_cents: rateCents,
						is_salaried: value.isSalaried,
						monthly_cost_cents: value.isSalaried ? monthlyCents : null,
						weekly_contract_minutes: 0,
					},
				})
			}
			memberForm.reset()
			setCreateMemberDialogOpen(false)
		},
	})

	const handleCreateMemberDialogOpenChange = (open: boolean) => {
		setCreateMemberDialogOpen(open)
		if (!open) memberForm.reset()
	}

	const members = catalog.members.data?.data ?? []
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

	const rows: TeamMemberRow[] = members.map((member) => {
		const profile = profileByMember.get(member.id)
		return {
			id: member.id,
			displayName: member.display_name,
			access: accessState(member, pendingByMemberId.has(member.id)),
			hourlyRateCents: profile?.hourly_rate_cents ?? null,
			isSalaried: profile?.is_salaried ?? false,
			monthlyCostCents: profile?.monthly_cost_cents ?? null,
			// Computed by the backend rather than divided here: the same rule feeds
			// the profitability report, and two copies of it would drift.
			effectiveHourlyRateCents: profile?.effective_hourly_rate_cents ?? null,
			weeklyContractMinutes: profile?.weekly_contract_minutes ?? null,
		}
	})

	const pendingRows: PendingInvitationRow[] = invitations.map((invitation) => {
		const member = invitation.member_id
			? members.find((item) => item.id === invitation.member_id)
			: undefined
		return {
			id: invitation.id,
			memberName: member?.display_name ?? 'Personne inconnue',
			expiresAt: invitation.expires_at,
		}
	})

	const normalizedSearch = search.trim().toLowerCase()
	const filteredRows = rows.filter((row) =>
		row.displayName.toLowerCase().includes(normalizedSearch),
	)

	const isLoading =
		catalog.members.isLoading || catalog.employeeProfiles.isLoading

	// `employee-profiles` is gated on `member.manage`, a stricter permission
	// than the plain organization membership `members` itself checks — see
	// `libs/core/src/application/employee/mod.rs`. A 403 here is an expected
	// access boundary for a seat without that permission, not a page failure:
	// it must not join the fatal error banner below, and the rows built from
	// an empty profile map must say so rather than claim nobody has a
	// profile. See #371.
	const hrDataRestricted = isForbiddenError(catalog.employeeProfiles.error)

	const error =
		catalog.members.error ??
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

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			const member = members.find((item) => item.id === draft.id)
			if (member) {
				await updateMember.mutateAsync({
					path: { member_id: member.id },
					body: {
						last_name: draft.values.lastName.trim(),
						first_name: draft.values.firstName.trim() || null,
					},
				})

				const rateCents = eurosToCents(draft.values.hourlyRate)
				const monthlyCents = eurosToCents(draft.values.monthlyCost)
				const existingProfile = profileByMember.get(member.id)
				if (rateCents !== null || draft.values.isSalaried) {
					await upsertProfile.mutateAsync({
						path: { member_id: member.id },
						body: {
							hourly_rate_cents: rateCents,
							is_salaried: draft.values.isSalaried,
							monthly_cost_cents: draft.values.isSalaried ? monthlyCents : null,
							weekly_contract_minutes:
								existingProfile?.weekly_contract_minutes ?? 0,
						},
					})
				} else if (existingProfile) {
					await removeProfile.mutateAsync({
						path: { member_id: member.id },
					})
				}
			}
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<>
			<memberForm.Subscribe selector={(state) => state.values}>
				{(memberValues) => (
					<TeamListUI
						organizationName={organizationName}
						organizationSlug={organizationSlug}
						isLoading={isLoading}
						error={mutationErrorMessage(error)}
						hrDataRestricted={hrDataRestricted}
						members={filteredRows}
						search={search}
						onSearchChange={setSearch}
						createForm={{
							values: memberValues,
							isPending: createMember.isPending || upsertProfile.isPending,
							onChange: (patch) => {
								for (const key of Object.keys(
									patch,
								) as (keyof MemberFormValues)[]) {
									memberForm.setFieldValue(key, patch[key] ?? '')
								}
							},
							onSubmit: () => void memberForm.handleSubmit(),
						}}
						createMemberDialogOpen={createMemberDialogOpen}
						onCreateMemberDialogOpenChange={handleCreateMemberDialogOpenChange}
						draft={draft}
						isSaving={isSaving}
						onEdit={(row) => {
							const member = members.find((item) => item.id === row.id)
							if (!member) return
							setDraft({
								id: member.id,
								values: {
									lastName: member.last_name,
									firstName: member.first_name ?? '',
									hourlyRate: centsToEuros(row.hourlyRateCents),
									isSalaried: row.isSalaried,
									monthlyCost: centsToEuros(row.monthlyCostCents),
								},
							})
						}}
						onDraftChange={(values) =>
							setDraft((current) =>
								current ? { ...current, values } : current,
							)
						}
						onCancelEdit={() => setDraft(null)}
						onSaveEdit={handleSaveDraft}
						onDeleteMember={(row) =>
							deleteMember.mutateAsync({ path: { member_id: row.id } })
						}
						onInvite={(row) => {
							setInviteTarget({ id: row.id, name: row.displayName })
							setInviteToken(null)
						}}
						pendingInvitations={pendingRows}
						revokingInvitationId={revokingInvitationId}
						onRevokeInvitation={(invitationId) =>
							void handleRevokeInvitation(invitationId)
						}
					/>
				)}
			</memberForm.Subscribe>
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
	)
}

/**
 * An empty field means "rate not set", not "free". Collapsing it to 0 would
 * feed a wrong cost into the profitability computation instead of an absent
 * one, which is the whole reason the column is nullable.
 */
function eurosToCents(value: string): number | null {
	const normalized = value.replace(',', '.').trim()
	if (normalized === '') {
		return null
	}
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) {
		return null
	}
	return Math.round(parsed * 100)
}

function centsToEuros(value: number | null): string {
	if (value === null) {
		return ''
	}
	return (value / 100).toFixed(2).replace('.', ',')
}

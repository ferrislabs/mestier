import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateMember,
	useDeleteMember,
	useReferenceCatalog,
	useRemoveEmployeeProfile,
	useUpdateMember,
	useUpsertEmployeeProfile,
} from '#/hooks/use-reference-catalog'
import { accessState, type MemberFormValues } from '#/pages/hr/types'
import {
	type MemberDraft,
	type TeamMemberRow,
	TeamListUI,
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

	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<MemberDraft | null>(null)
	const [isSaving, setIsSaving] = useState(false)

	const memberForm = useForm({
		defaultValues: {
			lastName: '',
			firstName: '',
			hourlyRate: '',
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
			if (rateCents !== null) {
				await upsertProfile.mutateAsync({
					path: { member_id: created.data.id },
					body: { hourly_rate_cents: rateCents, weekly_contract_minutes: 0 },
				})
			}
			memberForm.reset()
		},
	})

	const members = catalog.members.data?.data ?? []
	const profileByMember = new Map(
		(catalog.employeeProfiles.data?.data ?? []).map((profile) => [
			profile.member_id,
			profile,
		]),
	)

	const rows: TeamMemberRow[] = members.map((member) => {
		const profile = profileByMember.get(member.id)
		return {
			id: member.id,
			displayName: member.display_name,
			access: accessState(member),
			hourlyRateCents: profile?.hourly_rate_cents ?? null,
			weeklyContractMinutes: profile?.weekly_contract_minutes ?? null,
		}
	})

	const normalizedSearch = search.trim().toLowerCase()
	const filteredRows = rows.filter((row) =>
		row.displayName.toLowerCase().includes(normalizedSearch),
	)

	const isLoading =
		catalog.members.isLoading || catalog.employeeProfiles.isLoading

	const error =
		catalog.members.error ??
		catalog.employeeProfiles.error ??
		createMember.error ??
		updateMember.error ??
		deleteMember.error ??
		upsertProfile.error ??
		removeProfile.error

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
				const existingProfile = profileByMember.get(member.id)
				if (rateCents !== null) {
					await upsertProfile.mutateAsync({
						path: { member_id: member.id },
						body: {
							hourly_rate_cents: rateCents,
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
		<memberForm.Subscribe selector={(state) => state.values}>
			{(memberValues) => (
				<TeamListUI
					organizationName={organizationName}
					organizationSlug={organizationSlug}
					isLoading={isLoading}
					error={error?.message ?? null}
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
							},
						})
					}}
					onDraftChange={(values) =>
						setDraft((current) => (current ? { ...current, values } : current))
					}
					onCancelEdit={() => setDraft(null)}
					onSaveEdit={handleSaveDraft}
					onDeleteMember={(row) =>
						deleteMember.mutateAsync({ path: { member_id: row.id } })
					}
				/>
			)}
		</memberForm.Subscribe>
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

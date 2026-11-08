import { Plus } from 'lucide-react'
import type { ReactNode } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	DEFAULT_FILTERS,
	type MemberFilters,
	membersSummary,
} from '#/pages/hr/lib/rh/members-filter'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { DeadlinesCard } from '#/pages/hr/ui/rh/members/deadlines-card'
import type { MemberRealInfo } from '#/pages/hr/ui/rh/members/member-access'
import { MembersTable } from '#/pages/hr/ui/rh/members/members-table'
import { MembersToolbar } from '#/pages/hr/ui/rh/members/members-toolbar'
import {
	type PendingInvitationRow,
	PendingInvitations,
} from '#/pages/hr/ui/rh/members/pending-invitations'
import { useRhOrgSlug } from '#/pages/hr/ui/rh/shared/rh-context'

export interface MembersPageProps {
	state: RhState
	members: readonly Member[]
	today: IsoDate
	filters: MemberFilters
	onFiltersChange: (filters: MemberFilters) => void
	error: string | null
	hrDataRestricted: boolean
	realInfo: ReadonlyMap<string, MemberRealInfo>
	pendingInvitations: PendingInvitationRow[]
	revokingInvitationId: string | null
	renderRoles: (memberId: string) => ReactNode
	onCreate: () => void
	onEdit: (member: Member) => void
	onInvite: (member: Member) => void
	onDelete: (member: Member) => void
	onRevokeInvitation: (invitationId: string) => void
	dialogs: ReactNode
}

export function MembersPage({
	state,
	members,
	today,
	filters,
	onFiltersChange,
	error,
	hrDataRestricted,
	realInfo,
	pendingInvitations,
	revokingInvitationId,
	renderRoles,
	onCreate,
	onEdit,
	onInvite,
	onDelete,
	onRevokeInvitation,
	dialogs,
}: MembersPageProps) {
	const organizationSlug = useRhOrgSlug()
	const filtered =
		filters.query.trim() !== '' ||
		filters.contract !== DEFAULT_FILTERS.contract ||
		filters.status !== DEFAULT_FILTERS.status

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Membres"
				description={membersSummary(state.members)}
				actions={
					<RequirePermission permission="MANAGE_MEMBERS">
						<Button size="sm" onClick={onCreate}>
							<Plus aria-hidden /> Nouveau membre
						</Button>
					</RequirePermission>
				}
			/>

			{error ? (
				<div className="mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{!error && hrDataRestricted ? (
				<div className="mb-4 rounded-lg border border-border bg-muted/50 px-4 py-3 text-sm text-muted-foreground">
					Vous n’avez pas la permission de consulter les taux horaires et bases
					contractuelles de l’équipe.
				</div>
			) : null}

			<DeadlinesCard
				state={state}
				today={today}
				organizationSlug={organizationSlug}
			/>
			<MembersToolbar filters={filters} onChange={onFiltersChange} />
			<PendingInvitations
				invitations={pendingInvitations}
				revokingId={revokingInvitationId}
				onRevoke={onRevokeInvitation}
			/>
			{members.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucun membre ne correspond.</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Vérifie l’orthographe, le statut ou le type de contrat sélectionné.
					</p>
					{filtered && (
						<Button
							variant="outline"
							size="sm"
							className="mt-4"
							onClick={() => onFiltersChange(DEFAULT_FILTERS)}
						>
							Réinitialiser les filtres
						</Button>
					)}
				</div>
			) : (
				<MembersTable
					state={state}
					members={members}
					today={today}
					realInfo={realInfo}
					hrDataRestricted={hrDataRestricted}
					renderRoles={renderRoles}
					onEdit={onEdit}
					onInvite={onInvite}
					onDelete={onDelete}
				/>
			)}
			{dialogs}
		</PageShell>
	)
}

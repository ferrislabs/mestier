import { Loader2, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { formatDateFr } from '#/pages/hr/types'

export interface PendingInvitationRow {
	id: string
	memberName: string
	expiresAt: string
}

export function PendingInvitations({
	invitations,
	revokingId,
	onRevoke,
}: {
	invitations: PendingInvitationRow[]
	revokingId: string | null
	onRevoke: (invitationId: string) => void
}) {
	if (invitations.length === 0) return null

	return (
		<SectionCard className="mb-3">
			<SectionHeader
				title={`Invitations en attente (${invitations.length})`}
				description="Liens générés mais pas encore acceptés."
			/>
			<ul className="divide-y">
				{invitations.map((invitation) => (
					<li
						key={invitation.id}
						className="flex items-center justify-between px-5 py-3"
					>
						<div>
							<p className="font-medium">{invitation.memberName}</p>
							<p className="text-xs text-muted-foreground">
								Expire le {formatDateFr(invitation.expiresAt.slice(0, 10))}
							</p>
						</div>
						<Button
							variant="ghost"
							size="sm"
							onClick={() => onRevoke(invitation.id)}
							disabled={revokingId === invitation.id}
						>
							{revokingId === invitation.id ? (
								<Loader2 className="animate-spin" />
							) : (
								<X />
							)}
							Révoquer
						</Button>
					</li>
				))}
			</ul>
		</SectionCard>
	)
}

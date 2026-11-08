import { Link } from '@tanstack/react-router'
import {
	Archive,
	ArchiveRestore,
	ArrowLeft,
	CalendarPlus,
	Clock,
	Wallet,
} from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { mutationErrorMessage } from '#/lib/api-error'
import { contractCountLabel } from '#/pages/hr/lib/rh/format'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { MemberTab } from '#/pages/hr/lib/rh/rubriques'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { ArchiveDialog } from '#/pages/hr/ui/rh/members/archive-dialog'
import {
	ArchivedBadge,
	MemberContractBadge,
} from '#/pages/hr/ui/rh/members/contract-badge'
import { RubriqueList } from '#/pages/hr/ui/rh/members/rubrique-list'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import {
	useRhOrgPath,
	useRhProfileActions,
} from '#/pages/hr/ui/rh/shared/rh-context'

export function BackLink() {
	const orgPath = useRhOrgPath()
	return (
		<Link
			to={orgPath('/rh/membres')}
			activeOptions={{ exact: true }}
			className="-ml-1 inline-flex items-center gap-1.5 rounded-sm px-1 text-[13px] text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
		>
			<ArrowLeft className="size-4" aria-hidden />
			Membres
		</Link>
	)
}

export function MemberSidebar({
	state,
	member,
	today,
	tab,
}: {
	state: RhState
	member: Member
	today: IsoDate
	tab: MemberTab
}) {
	const profiles = useRhProfileActions()
	const orgPath = useRhOrgPath()
	const [archiving, setArchiving] = useState(false)
	const [pending, setPending] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const run = async (action: () => Promise<void>) => {
		setPending(true)
		setError(null)
		try {
			await action()
		} catch (failure) {
			setError(mutationErrorMessage(failure))
		} finally {
			setPending(false)
			setArchiving(false)
		}
	}
	const archived = member.status === 'archived'
	return (
		<aside
			aria-label="Fiche du membre"
			className="flex flex-col border-border border-b bg-card lg:min-h-0 lg:border-r lg:border-b-0"
		>
			<div className="shrink-0 border-border border-b px-4 py-4">
				<BackLink />
				<div className="mt-3 flex items-center gap-3">
					<MemberAvatar member={member} size="xl" />
					<div className="min-w-0">
						<h1 className="truncate font-semibold text-lg leading-tight tracking-tight">
							{fullName(member)}
						</h1>
						<p className="truncate text-muted-foreground text-sm">
							{member.jobTitle || 'Poste non renseigné'}
						</p>
						<div className="mt-1.5 flex flex-wrap items-center gap-1.5">
							<MemberContractBadge member={member} today={today} />
							{member.contracts.length > 1 && (
								<span className="text-[12px] text-muted-foreground">
									{contractCountLabel(member.contracts.length)}
								</span>
							)}
							{archived && <ArchivedBadge />}
						</div>
					</div>
				</div>
				<div className="mt-3 flex flex-wrap items-center gap-2">
					{!archived && (
						<Button variant="outline" size="sm" asChild>
							<Link
								to={orgPath('/rh/absences/nouvelle')}
								search={{ membre: member.id }}
							>
								<CalendarPlus aria-hidden /> Poser une absence
							</Link>
						</Button>
					)}
					<RequirePermission permission="MANAGE_MEMBERS">
						{archived ? (
							<Button
								variant="outline"
								size="sm"
								disabled={pending}
								onClick={() => void run(() => profiles.restore(member.id))}
							>
								<ArchiveRestore aria-hidden /> Réactiver
							</Button>
						) : (
							<ArchiveDialog
								state={state}
								member={member}
								today={today}
								open={archiving}
								onOpenChange={setArchiving}
								onConfirm={() => void run(() => profiles.archive(member.id))}
								trigger={
									<Button variant="outline" size="sm" disabled={pending}>
										<Archive aria-hidden /> Archiver
									</Button>
								}
							/>
						)}
					</RequirePermission>
				</div>
				{error && (
					<p role="alert" className="mt-2 text-[12px] text-state-alert">
						{error}
					</p>
				)}
				<div className="mt-2 flex flex-wrap items-center gap-2">
					<Button variant="ghost" size="sm" asChild>
						<Link
							to={orgPath('/rh/membres/$memberId/temps-de-travail')}
							params={{ memberId: member.id }}
						>
							<Clock aria-hidden /> Temps de travail
						</Link>
					</Button>
					<RequirePermission permission="MANAGE_MEMBERS">
						<Button variant="ghost" size="sm" asChild>
							<Link
								to={orgPath('/rh/membres/$memberId/temps-de-travail')}
								params={{ memberId: member.id }}
								hash="historique-cout"
							>
								<Wallet aria-hidden /> Historique du coût
							</Link>
						</Button>
					</RequirePermission>
				</div>
			</div>
			<RubriqueList
				state={state}
				member={member}
				today={today}
				selected={tab}
			/>
		</aside>
	)
}

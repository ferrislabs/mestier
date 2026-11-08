import { Link } from '@tanstack/react-router'
import { CalendarPlus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { absenceDays, findType } from '#/pages/hr/lib/rh/absences'
import { computeBalance } from '#/pages/hr/lib/rh/balances'
import { formatDays, formatRange } from '#/pages/hr/lib/rh/dates'
import { absenceLists } from '#/pages/hr/lib/rh/rubriques'
import type { Absence, IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { DetailCard, EmptyNote } from '#/pages/hr/ui/rh/members/detail-card'
import { AbsenceStatusBadge } from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

const LINK =
	'rounded-sm text-[13px] text-primary underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring'

function AbsenceList({
	label,
	absences,
	state,
	member,
	empty,
}: {
	label: string
	absences: readonly Absence[]
	state: RhState
	member: Member
	empty: string
}) {
	return (
		<section aria-label={label} className="grid gap-1.5">
			<h4 className="text-[12px] text-muted-foreground">{label}</h4>
			{absences.length === 0 ? (
				<EmptyNote>{empty}</EmptyNote>
			) : (
				<ul className="-mx-1 grid divide-y">
					{absences.map((absence) => {
						const type = findType(state, absence.typeId)
						return (
							<li
								key={absence.id}
								className="flex flex-wrap items-center gap-x-3 gap-y-1 px-1 py-2 first:pt-0 last:pb-0"
							>
								<TypeChip type={type ?? { name: 'Absence', hue: 250 }} />
								<span className="min-w-0 flex-1 text-[13px]">
									{formatRange(absence.startDate, absence.endDate)}
									{type && (
										<span className="ml-2 text-muted-foreground">
											{formatDays(absenceDays(absence, member, type))}
										</span>
									)}
								</span>
								<AbsenceStatusBadge status={absence.status} />
							</li>
						)
					})}
				</ul>
			)}
		</section>
	)
}

export function AbsencesCard({
	state,
	member,
	today,
}: {
	state: RhState
	member: Member
	today: IsoDate
}) {
	const orgPath = useRhOrgPath()
	const { upcoming, past } = absenceLists(state, member.id, today)
	return (
		<DetailCard
			title="Absences"
			action={
				<Link
					to={orgPath('/rh/absences')}
					search={{ membre: member.id }}
					className={LINK}
				>
					Voir tout
				</Link>
			}
		>
			<div className="grid gap-5">
				<section aria-label="Soldes" className="grid gap-1.5">
					<h4 className="text-[12px] text-muted-foreground">Soldes</h4>
					{state.accounts.length === 0 ? (
						<EmptyNote>Aucun compte de congés n’est configuré.</EmptyNote>
					) : (
						<>
							<div className="grid grid-cols-[1fr_6rem_6rem] items-baseline gap-x-3 gap-y-1.5 text-[13px]">
								<span />
								<span className="text-right text-[12px] text-muted-foreground">
									Solde
								</span>
								<span className="text-right text-[12px] text-muted-foreground">
									Disponible
								</span>
								{state.accounts.map((account) => {
									const line = computeBalance(state, member, account, today)
									return (
										<div key={account.id} className="contents">
											<Link
												to={orgPath('/rh/droits')}
												search={{ compte: account.id }}
												className="w-fit rounded-sm font-medium outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
											>
												{account.name}
											</Link>
											<span className="tabular text-right">
												{formatDays(line.balance)}
											</span>
											<span
												className={cn(
													'tabular text-right',
													line.available < 0 && 'text-state-alert',
												)}
											>
												{formatDays(line.available)}
											</span>
										</div>
									)
								})}
							</div>
							<p className="text-[12px] text-muted-foreground">
								Le disponible retire du solde les jours déjà planifiés ou en
								attente de validation.
							</p>
						</>
					)}
				</section>
				<AbsenceList
					label="À venir"
					absences={upcoming}
					state={state}
					member={member}
					empty="Aucune absence en cours ou à venir."
				/>
				<AbsenceList
					label="Passées"
					absences={past}
					state={state}
					member={member}
					empty="Aucune absence passée."
				/>
				{member.status === 'active' && (
					<div className="border-t pt-3">
						<Button variant="outline" size="sm" asChild>
							<Link
								to={orgPath('/rh/absences/nouvelle')}
								search={{ membre: member.id }}
							>
								<CalendarPlus aria-hidden /> Poser une absence
							</Link>
						</Button>
					</div>
				)}
			</div>
		</DetailCard>
	)
}

import { ChevronRight } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Table,
	TableBody,
	TableCell,
	TableFooter,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import type { BalanceLine } from '#/pages/hr/lib/rh/balances'
import { fullName } from '#/pages/hr/lib/rh/members'
import {
	acquiredTotal,
	consumptionOf,
	describeBreakdown,
	formatAmount,
	LOW_BALANCE_THRESHOLD,
	type RightsRow,
	type RightsTotals,
} from '#/pages/hr/lib/rh/rights-model'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'

function Amount({
	value,
	dashWhenZero = false,
	className,
}: {
	value: number
	dashWhenZero?: boolean
	className?: string
}) {
	if (dashWhenZero && value === 0)
		return <span className="text-muted-foreground/60">—</span>
	return (
		<span className={cn(value < 0 && 'text-state-alert', className)}>
			{formatAmount(value)}
		</span>
	)
}

function ConsumptionBar({ line }: { line: BalanceLine }) {
	const { total, takenShare, plannedShare } = consumptionOf(line)
	if (total === 0 || line.taken + line.planned === 0) return null
	return (
		<span
			role="img"
			aria-label={`Consommation de la période : ${formatAmount(line.taken)} j pris et ${formatAmount(line.planned)} j posés sur ${formatAmount(total)} j`}
			className="mt-2 ml-auto flex h-1 w-14 overflow-hidden rounded-full bg-muted"
		>
			<span
				className="bg-state-progress"
				style={{ width: `${takenShare * 100}%` }}
			/>
			<span
				className="bg-state-progress/35"
				style={{ width: `${plannedShare * 100}%` }}
			/>
		</span>
	)
}

function availableTone(value: number): string | undefined {
	if (value < 0) return undefined
	return value < LOW_BALANCE_THRESHOLD ? 'text-state-today' : undefined
}

export function BalancesTable({
	rows,
	totals,
	onOpen,
}: {
	rows: readonly RightsRow[]
	totals: RightsTotals
	onOpen: (memberId: string) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Membre</TableHead>
						<TableHead
							className="text-right"
							title="Acquis à ce jour, report et régularisations inclus"
						>
							Acquis
						</TableHead>
						<TableHead
							className="text-right"
							title="Absences validées jusqu’à aujourd’hui"
						>
							Pris
						</TableHead>
						<TableHead className="text-right" title="Absences validées à venir">
							Posés
						</TableHead>
						<TableHead className="text-right" title="Demandes à valider">
							En attente
						</TableHead>
						<TableHead className="text-right" title="Acquis moins pris">
							Solde
						</TableHead>
						<TableHead
							className="text-right"
							title="Solde moins posés et en attente"
						>
							Disponible
						</TableHead>
						<TableHead
							className="text-right"
							title="Prévision une fois toute la période acquise, après tout ce qui est pris, posé ou en attente"
						>
							Fin de période
						</TableHead>
						<TableHead className="w-28 pr-4">
							<span className="sr-only">Détail</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{rows.map(({ member, line }) => {
						const detail = describeBreakdown(line)
						return (
							<TableRow key={member.id} className="relative cursor-pointer">
								<TableCell className="py-2.5 pl-4">
									<div className="flex items-center gap-3">
										<MemberAvatar member={member} size="lg" />
										<span className="min-w-0">
											<span className="block truncate font-medium">
												{fullName(member)}
											</span>
											<span className="block text-[12px] text-muted-foreground">
												{member.jobTitle}
											</span>
										</span>
									</div>
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={acquiredTotal(line)} />
									{detail && (
										<span className="block text-[12px] text-muted-foreground">
											{detail}
										</span>
									)}
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={line.taken} dashWhenZero />
									<ConsumptionBar line={line} />
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={line.planned} dashWhenZero />
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={line.pending} dashWhenZero />
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={line.balance} />
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right font-semibold">
									<Amount
										value={line.available}
										className={availableTone(line.available)}
									/>
								</TableCell>
								<TableCell className="tabular py-2.5 align-top text-right">
									<Amount value={line.forecast} />
								</TableCell>
								<TableCell className="pr-4 text-right">
									<Button
										variant="ghost"
										size="sm"
										aria-label={`Détail de ${fullName(member)}`}
										className="text-muted-foreground after:absolute after:inset-0"
										onClick={() => onOpen(member.id)}
									>
										Détail <ChevronRight />
									</Button>
								</TableCell>
							</TableRow>
						)
					})}
					{rows.length === 0 && (
						<TableRow className="hover:bg-transparent">
							<TableCell
								colSpan={9}
								className="py-10 text-center text-muted-foreground"
							>
								Aucun membre actif sur cette période.
							</TableCell>
						</TableRow>
					)}
				</TableBody>
				{rows.length > 0 && (
					<TableFooter>
						<TableRow className="hover:bg-transparent">
							<TableCell className="py-3 pl-4">
								Total équipe
								<span className="ml-2 font-normal text-[12px] text-muted-foreground">
									{totals.count} {totals.count > 1 ? 'membres' : 'membre'}
								</span>
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.acquired} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.taken} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.planned} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.pending} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.balance} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.available} />
							</TableCell>
							<TableCell className="tabular text-right">
								<Amount value={totals.forecast} />
							</TableCell>
							<TableCell className="pr-4" />
						</TableRow>
					</TableFooter>
				)}
			</Table>
		</div>
	)
}

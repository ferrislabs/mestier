import { Link } from '@tanstack/react-router'
import { ArrowRight, Clock, Loader2, Users } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import type { AbsenceKind } from '#/pages/hr/lib/absences'
import { ABSENCE_KIND_LABELS } from '#/pages/hr/lib/absences'
import { formatDateFr, formatDurationMinutes } from '#/pages/hr/types'

export interface WorkTimeOverviewNextAbsence {
	/** A bare `YYYY-MM-DD` local calendar date — already resolved from the absence's instant. */
	date: string
	kind: AbsenceKind
}

export interface WorkTimeOverviewRow {
	memberId: string
	displayName: string
	/** `null` when the seat has no employee profile yet. */
	weeklyContractMinutes: number | null
	/** The soonest absence starting today or later, or `null` when none is scheduled. */
	nextAbsence: WorkTimeOverviewNextAbsence | null
}

interface WorkTimeOverviewUIProps {
	organizationName: string
	organizationSlug: string
	isLoading: boolean
	error: string | null
	/**
	 * True when the caller lacks the permission to read employee profiles
	 * (`member.manage`) — an expected access boundary, not a failure. Every
	 * row's `weeklyContractMinutes` is `null` in that case regardless of
	 * whether the seat actually has a profile, so the table says "not
	 * visible to you" rather than the misleading "no profile" it would
	 * otherwise show for everyone. See #371.
	 */
	hrDataRestricted: boolean
	rows: WorkTimeOverviewRow[]
}

export function WorkTimeOverviewUI({
	organizationName,
	organizationSlug,
	isLoading,
	error,
	hrDataRestricted,
	rows,
}: WorkTimeOverviewUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Temps de travail"
				description="Base contractuelle hebdomadaire et prochaine absence connue de chaque personne."
			/>

			<MetricCard
				label="Équipe"
				value={rows.length}
				hint="Personnes suivies"
				icon={<Users className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{!error && hrDataRestricted ? (
				<div className="rounded-lg border border-border bg-muted/50 px-4 py-3 text-sm text-muted-foreground">
					Vous n’avez pas la permission de consulter les bases contractuelles de
					l’équipe.
				</div>
			) : null}

			{isLoading ? (
				<WorkTimeOverviewUI.Loading />
			) : (
				<WorkTimeTable
					rows={rows}
					organizationSlug={organizationSlug}
					hrDataRestricted={hrDataRestricted}
				/>
			)}
		</PageShell>
	)
}

WorkTimeOverviewUI.Loading = function WorkTimeOverviewLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement du temps de travail…
			</SectionCard>
		</PageShell>
	)
}

interface WorkTimeTableProps {
	rows: WorkTimeOverviewRow[]
	organizationSlug: string
	hrDataRestricted: boolean
}

function WorkTimeTable({
	rows,
	organizationSlug,
	hrDataRestricted,
}: WorkTimeTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Équipe (${rows.length})`}
				description="Base contractuelle et prochaine absence connue de chaque personne."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[640px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Nom
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Base contractuelle
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Prochaine absence
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{rows.length === 0 ? (
							<tr>
								<td colSpan={4} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucune personne trouvée</p>
										<p className="text-sm text-muted-foreground">
											Ajoutez des personnes dans l’équipe pour suivre leur temps
											de travail ici.
										</p>
									</div>
								</td>
							</tr>
						) : (
							rows.map((row) => (
								<tr
									key={row.memberId}
									className="border-b transition hover:bg-muted/35 last:border-b-0"
								>
									<td className="px-5 py-3 align-middle">
										<p className="truncate font-medium">{row.displayName}</p>
									</td>
									<td className="px-5 py-3 align-middle">
										{row.weeklyContractMinutes === null ? (
											<span className="text-muted-foreground italic">
												{hrDataRestricted
													? 'Non consultable'
													: 'Sans profil RH'}
											</span>
										) : (
											<span className="font-medium tabular-nums">
												{formatDurationMinutes(row.weeklyContractMinutes)}
												<span className="text-muted-foreground">/sem.</span>
											</span>
										)}
									</td>
									<td className="px-5 py-3 align-middle">
										{row.nextAbsence ? (
											<div className="flex items-center gap-2">
												<span className="tabular-nums">
													{formatDateFr(row.nextAbsence.date)}
												</span>
												<StatusBadge tone="neutral">
													{ABSENCE_KIND_LABELS[row.nextAbsence.kind]}
												</StatusBadge>
											</div>
										) : (
											<span className="text-muted-foreground">—</span>
										)}
									</td>
									<td className="px-5 py-3 align-middle">
										<div className="flex justify-end">
											<Button variant="ghost" size="sm" asChild>
												<Link
													to={buildOrgPath(
														organizationSlug,
														'/hr/team/$memberId/work-time',
													)}
													params={{ memberId: row.memberId }}
												>
													<Clock />
													Détails
													<ArrowRight />
												</Link>
											</Button>
										</div>
									</td>
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

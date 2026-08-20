import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowUpRight,
	Clock,
	Euro,
	TrendingDown,
	TrendingUp,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import type {
	EmployeeProfitability,
	JobProfitability,
	Period,
} from '#/hooks/use-reporting'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	formatCents,
	formatMarginRate,
	formatMinutes,
	incompleteReason,
	isCompleteJob,
	realCostCents,
	recollectedNote,
} from '#/pages/reporting/types'

interface ProfitabilityUIProps {
	period: Period
	organizationSlug: string
	jobs: JobProfitability[]
	mostProfitable: JobProfitability[]
	leastProfitable: JobProfitability[]
	incomplete: JobProfitability[]
	employees: EmployeeProfitability[]
	/** Resolves an employee id into a name, or `null` while unknown. */
	employeeName: (employeeId: string) => string | null
	isLoading: boolean
	error: string | null
	onPeriodChange: (period: Period) => void
	onRetry: () => void
}

/**
 * The payoff screen: what each projet cost against what it was quoted.
 *
 * Presentational only. Every figure here was computed by the backend, including
 * the rankings, so this file never decides what "least profitable" means.
 */
export function ProfitabilityUI({
	period,
	organizationSlug,
	jobs,
	mostProfitable,
	leastProfitable,
	incomplete,
	employees,
	employeeName,
	isLoading,
	error,
	onPeriodChange,
	onRetry,
}: ProfitabilityUIProps) {
	// Only the jobs with complete figures, so an incomplete one — cost is a
	// floor, margin unstated — cannot flatter or spoil a total that reads as
	// trustworthy. The three headline tiles below share this exact rule.
	const completeJobs = jobs.filter(isCompleteJob)
	const quoted = completeJobs.reduce(
		(sum, job) => sum + (job.quoted_cents ?? 0),
		0,
	)
	const cost = completeJobs.reduce((sum, job) => sum + realCostCents(job), 0)
	const margin = completeJobs.reduce(
		(sum, job) => sum + (job.margin_cents ?? 0),
		0,
	)
	const totalWorkedMinutes = employees.reduce(
		(sum, employee) => sum + employee.worked_minutes,
		0,
	)

	return (
		<PageShell>
			<PageHeader
				title="Rentabilité"
				description="Ce que chaque projet a coûté, comparé à ce qui a été devisé."
				actions={
					<div className="flex flex-wrap items-end gap-2">
						<div className="space-y-1">
							<Label htmlFor="from" className="text-xs text-muted-foreground">
								Du
							</Label>
							<Input
								id="from"
								type="date"
								className="w-40"
								value={period.from}
								onChange={(event) =>
									onPeriodChange({ ...period, from: event.target.value })
								}
							/>
						</div>
						<div className="space-y-1">
							<Label htmlFor="to" className="text-xs text-muted-foreground">
								Au
							</Label>
							<Input
								id="to"
								type="date"
								className="w-40"
								value={period.to}
								onChange={(event) =>
									onPeriodChange({ ...period, to: event.target.value })
								}
							/>
						</div>
					</div>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-4">
				<MetricCard
					label="Devisé"
					value={formatCents(quoted)}
					hint="Projets complets uniquement"
					icon={<Euro className="size-4" />}
				/>
				<MetricCard
					label="Coût réel"
					value={formatCents(cost)}
					hint="Main d'œuvre et matériel, projets complets uniquement"
				/>
				<MetricCard
					label="Marge"
					value={formatCents(margin)}
					hint="Projets complets uniquement"
				/>
				<MetricCard
					label="Heures travaillées"
					value={formatMinutes(totalWorkedMinutes)}
					icon={<Clock className="size-4" />}
				/>
			</section>

			{incomplete.length > 0 ? (
				<SectionCard className="border-amber-500/30 bg-amber-50 p-5 dark:bg-amber-950/20">
					<div className="flex items-start gap-3">
						<AlertCircle className="mt-0.5 size-5 shrink-0 text-amber-600 dark:text-amber-500" />
						<div className="min-w-0 space-y-2">
							<p className="text-sm font-semibold">
								{incomplete.length} projet
								{incomplete.length > 1 ? 's' : ''} sans marge calculable
							</p>
							<p className="text-sm text-muted-foreground">
								Leur coût est un minimum, pas un total. Aucune marge n'est
								affichée pour eux, plutôt qu'une marge fausse.
							</p>
							<ul className="space-y-1 text-sm">
								{incomplete.map((job) => (
									<li key={job.task_id} className="flex flex-wrap gap-x-2">
										<JobTitleLink
											title={job.title}
											organizationSlug={organizationSlug}
											className="font-medium"
										/>
										<span className="text-muted-foreground">
											{incompleteReason(job)}
										</span>
									</li>
								))}
							</ul>
						</div>
					</div>
				</SectionCard>
			) : null}

			<div className="grid gap-4 lg:grid-cols-2">
				<RankingCard
					title="Les plus rentables"
					icon={<TrendingUp className="size-4 text-primary" />}
					jobs={mostProfitable}
					organizationSlug={organizationSlug}
				/>
				<RankingCard
					title="Les moins rentables"
					icon={<TrendingDown className="size-4 text-destructive" />}
					jobs={leastProfitable}
					organizationSlug={organizationSlug}
				/>
			</div>

			<SectionCard>
				<SectionHeader
					title={`Projets (${jobs.length})`}
					description="Coût réel, marge et temps passé sur la période."
				/>
				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : jobs.length === 0 ? (
					<p className="p-5 text-sm text-muted-foreground">
						Aucun temps pointé sur cette période. Un projet n'apparaît ici
						qu'à partir du moment où quelqu'un y a travaillé.
					</p>
				) : (
					<ul className="divide-y">
						{jobs.map((job) => (
							<li
								key={job.task_id}
								className="grid gap-2 px-5 py-4 sm:grid-cols-[minmax(0,1fr)_repeat(4,110px)] sm:items-center"
							>
								<div className="min-w-0">
									<JobTitleLink
										title={job.title}
										organizationSlug={organizationSlug}
										className="font-medium"
									/>
									{incompleteReason(job) ? (
										<p className="truncate text-xs text-amber-600 dark:text-amber-500">
											{incompleteReason(job)}
										</p>
									) : null}
									{recollectedNote(job) ? (
										<p className="truncate text-xs text-muted-foreground">
											{recollectedNote(job)}
										</p>
									) : null}
								</div>
								<Figure
									label="Devisé"
									value={
										job.quoted_cents === null || job.quoted_cents === undefined
											? '—'
											: formatCents(job.quoted_cents)
									}
								/>
								<Figure label="Coût" value={formatCents(realCostCents(job))} />
								<Figure
									label="Marge"
									value={
										job.margin_cents === null || job.margin_cents === undefined
											? '—'
											: formatCents(job.margin_cents)
									}
									strong
								/>
								<Figure
									label="Temps"
									value={formatMinutes(job.worked_minutes)}
								/>
							</li>
						))}
					</ul>
				)}
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title="Par salarié"
					description="Heures pointées et coût sur la période, pour la paie comme pour le suivi."
				/>
				{employees.length === 0 ? (
					<p className="p-5 text-sm text-muted-foreground">
						Aucun pointage sur cette période.
					</p>
				) : (
					<ul className="divide-y">
						{employees.map((row) => (
							<li
								key={row.employee_id}
								className="flex flex-wrap items-center justify-between gap-3 px-5 py-4"
							>
								<div className="min-w-0">
									<p className="truncate font-medium">
										{employeeName(row.employee_id) ?? 'Salarié'}
									</p>
									{row.rate_missing ? (
										<p className="text-xs text-amber-600 dark:text-amber-500">
											Taux horaire manquant, coût non calculé
										</p>
									) : null}
									{row.open_entries > 0 ? (
										<p className="text-xs text-amber-600 dark:text-amber-500">
											{row.open_entries} pointage
											{row.open_entries > 1 ? 's' : ''} non clôturé
											{row.open_entries > 1 ? 's' : ''}, ce total est incomplet
										</p>
									) : null}
								</div>
								<div className="shrink-0 text-right">
									<p className="font-semibold tabular-nums">
										{formatMinutes(row.worked_minutes)}
									</p>
									<p className="text-xs text-muted-foreground tabular-nums">
										{formatCents(row.labour_cost_cents)}
									</p>
								</div>
							</li>
						))}
					</ul>
				)}
			</SectionCard>
		</PageShell>
	)
}

function RankingCard({
	title,
	icon,
	jobs,
	organizationSlug,
}: {
	title: string
	icon: React.ReactNode
	jobs: JobProfitability[]
	organizationSlug: string
}) {
	return (
		<SectionCard>
			<SectionHeader title={title} />
			{jobs.length === 0 ? (
				<p className="p-5 text-sm text-muted-foreground">
					Aucun projet n'a de marge calculable sur cette période.
				</p>
			) : (
				<ul className="divide-y">
					{jobs.map((job) => (
						<li
							key={job.task_id}
							className="flex items-center justify-between gap-3 px-5 py-3"
						>
							<div className="flex min-w-0 items-center gap-2">
								{icon}
								<JobTitleLink
									title={job.title}
									organizationSlug={organizationSlug}
									className="truncate text-sm font-medium"
								/>
							</div>
							<div className="shrink-0 text-right">
								<p className="font-semibold tabular-nums">
									{job.margin_cents === null || job.margin_cents === undefined
										? '—'
										: formatCents(job.margin_cents)}
								</p>
								<p className="text-xs text-muted-foreground">
									{formatMarginRate(job)}
								</p>
							</div>
						</li>
					))}
				</ul>
			)}
		</SectionCard>
	)
}

/**
 * A job title that opens the Planning module's task list.
 *
 * Not a deep link to the exact task: neither an existing cross-module
 * deep-link pattern nor a `taskId` query param read by the task list exists
 * to open one directly (see the workstream report). Landing on the list is
 * still strictly better than plain text — it is where a manager would go
 * next to act on what this screen just told them.
 */
function JobTitleLink({
	title,
	organizationSlug,
	className,
}: {
	title: string
	organizationSlug: string
	className?: string
}) {
	return (
		<Link
			to={buildOrgPath(organizationSlug, '/planning/tasks')}
			title="Ouvrir la liste des tâches du planning pour retrouver ce projet"
			className={cn(
				'group inline-flex min-w-0 items-center gap-1 hover:underline',
				className,
			)}
		>
			<span className="truncate">{title}</span>
			<ArrowUpRight className="size-3 shrink-0 text-muted-foreground opacity-0 group-hover:opacity-100" />
		</Link>
	)
}

function Figure({
	label,
	value,
	strong,
}: {
	label: string
	value: string
	strong?: boolean
}) {
	return (
		<div className="sm:text-right">
			<p className="text-xs text-muted-foreground sm:hidden">{label}</p>
			<p className={strong ? 'font-semibold tabular-nums' : 'tabular-nums'}>
				{value}
			</p>
		</div>
	)
}

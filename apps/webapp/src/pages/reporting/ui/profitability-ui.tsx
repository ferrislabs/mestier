import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowUpRight,
	Building2,
	Clock,
	Euro,
	TrendingDown,
	TrendingUp,
	Users,
} from 'lucide-react'
import { Badge } from '#/components/ui/badge'
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
	MemberProfitability,
	Period,
	ProjectProfitability,
} from '#/hooks/use-reporting'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	expensesNote,
	formatCents,
	formatMarginRate,
	formatMinutes,
	incompleteReason,
	isCompleteProject,
	overlapNote,
	plannedCostCents,
} from '#/pages/reporting/types'

interface ProfitabilityUIProps {
	period: Period
	organizationSlug: string
	projects: ProjectProfitability[]
	mostProfitable: ProjectProfitability[]
	leastProfitable: ProjectProfitability[]
	incomplete: ProjectProfitability[]
	doubleBooked: ProjectProfitability[]
	members: MemberProfitability[]
	/** Resolves a member id into a name, or `null` while unknown. */
	memberName: (memberId: string) => string | null
	isLoading: boolean
	error: string | null
	onPeriodChange: (period: Period) => void
	onRetry: () => void
}

/**
 * The payoff screen: what each project is planned to cost against what it was
 * quoted.
 *
 * Presentational only. Every figure here was computed by the backend, including
 * the rankings, so this file never decides what "least profitable" means.
 */
export function ProfitabilityUI({
	period,
	organizationSlug,
	projects,
	mostProfitable,
	leastProfitable,
	incomplete,
	doubleBooked,
	members,
	memberName,
	isLoading,
	error,
	onPeriodChange,
	onRetry,
}: ProfitabilityUIProps) {
	const completeProjects = projects.filter(isCompleteProject)
	const quoted = completeProjects.reduce(
		(sum, project) => sum + (project.quoted_cents ?? 0),
		0,
	)
	const cost = completeProjects.reduce(
		(sum, project) => sum + plannedCostCents(project),
		0,
	)
	const margin = completeProjects.reduce(
		(sum, project) => sum + (project.margin_cents ?? 0),
		0,
	)
	const totalPlannedMinutes = members.reduce(
		(sum, member) => sum + member.planned_minutes,
		0,
	)

	return (
		<PageShell>
			<PageHeader
				title="Rentabilité"
				description="Ce que chaque projet coûte d'après le planning, comparé à ce qui a été devisé."
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
					label="Coût planifié"
					value={formatCents(cost)}
					hint="Main d'œuvre, matériel et frais, projets complets uniquement"
				/>
				<MetricCard
					label="Marge"
					value={formatCents(margin)}
					hint="Projets complets uniquement"
				/>
				<MetricCard
					label="Heures planifiées"
					value={formatMinutes(totalPlannedMinutes)}
					hint="D'après le planning, personne ne pointe"
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
								{incomplete.map((project) => (
									<li
										key={project.project_id}
										className="flex flex-wrap gap-x-2"
									>
										<ProjectTitleLink
											project={project}
											organizationSlug={organizationSlug}
											className="font-medium"
										/>
										<span className="text-muted-foreground">
											{incompleteReason(project)}
										</span>
									</li>
								))}
							</ul>
						</div>
					</div>
				</SectionCard>
			) : null}

			{doubleBooked.length > 0 ? (
				<SectionCard className="border-sky-500/30 bg-sky-50 p-5 dark:bg-sky-950/20">
					<div className="flex items-start gap-3">
						<Users className="mt-0.5 size-5 shrink-0 text-sky-600 dark:text-sky-500" />
						<div className="min-w-0 space-y-2">
							<p className="text-sm font-semibold">
								{doubleBooked.length} projet
								{doubleBooked.length > 1 ? 's' : ''} avec du temps compté deux
								fois
							</p>
							<p className="text-sm text-muted-foreground">
								Quelqu'un est affecté à deux tâches en même temps. Les chiffres
								ci-dessous sont exacts, c'est le planning qui est à corriger.
							</p>
							<ul className="space-y-1 text-sm">
								{doubleBooked.map((project) => (
									<li
										key={project.project_id}
										className="flex flex-wrap gap-x-2"
									>
										<ProjectTitleLink
											project={project}
											organizationSlug={organizationSlug}
											className="font-medium"
										/>
										<span className="text-muted-foreground">
											{formatMinutes(project.overlapping_minutes)} en double
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
					projects={mostProfitable}
					organizationSlug={organizationSlug}
				/>
				<RankingCard
					title="Les moins rentables"
					icon={<TrendingDown className="size-4 text-destructive" />}
					projects={leastProfitable}
					organizationSlug={organizationSlug}
				/>
			</div>

			<SectionCard>
				<SectionHeader
					title={`Projets (${projects.length})`}
					description="Coût planifié, marge et temps sur la période. Les projets internes y figurent aussi : une réunion coûte."
				/>
				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : projects.length === 0 ? (
					<p className="p-5 text-sm text-muted-foreground">
						Rien de planifié sur cette période. Un projet apparaît ici dès
						qu'une de ses tâches y tombe, sans attendre que quelqu'un pointe.
					</p>
				) : (
					<ul className="divide-y">
						{projects.map((project) => (
							<li
								key={project.project_id}
								className="grid gap-2 px-5 py-4 sm:grid-cols-[minmax(0,1fr)_repeat(5,100px)] sm:items-center"
							>
								<div className="min-w-0">
									<div className="flex min-w-0 flex-wrap items-center gap-2">
										<ProjectTitleLink
											project={project}
											organizationSlug={organizationSlug}
											className="font-medium"
										/>
										{project.customer_id ? null : (
											<Badge variant="secondary" className="gap-1">
												<Building2 className="size-3" />
												Interne
											</Badge>
										)}
									</div>
									{incompleteReason(project) ? (
										<p className="truncate text-xs text-amber-600 dark:text-amber-500">
											{incompleteReason(project)}
										</p>
									) : null}
									{overlapNote(project) ? (
										<p className="truncate text-xs text-sky-600 dark:text-sky-500">
											{overlapNote(project)}
										</p>
									) : null}
								</div>
								<Figure
									label="Devisé"
									value={
										project.quoted_cents === null ||
										project.quoted_cents === undefined
											? '—'
											: formatCents(project.quoted_cents)
									}
								/>
								<Figure
									label="Coût"
									value={formatCents(plannedCostCents(project))}
								/>
								<Figure
									label="Frais"
									value={
										project.expenses_cents > 0
											? formatCents(project.expenses_cents)
											: '—'
									}
								/>
								<Figure
									label="Marge"
									value={
										project.margin_cents === null ||
										project.margin_cents === undefined
											? '—'
											: formatCents(project.margin_cents)
									}
									strong
								/>
								<Figure
									label="Temps"
									value={formatMinutes(project.planned_minutes)}
								/>
							</li>
						))}
					</ul>
				)}
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title="Par personne"
					description="Heures planifiées et coût sur la période, pour la paie comme pour le suivi. Ce sont les heures du planning, pas des pointages."
				/>
				{members.length === 0 ? (
					<p className="p-5 text-sm text-muted-foreground">
						Personne n'est affecté sur cette période.
					</p>
				) : (
					<ul className="divide-y">
						{members.map((row) => (
							<li
								key={row.member_id}
								className="flex flex-wrap items-center justify-between gap-3 px-5 py-4"
							>
								<div className="min-w-0">
									<p className="truncate font-medium">
										{memberName(row.member_id) ?? 'Membre'}
									</p>
									{row.rate_missing ? (
										<p className="text-xs text-amber-600 dark:text-amber-500">
											Coût horaire manquant : taux horaire ou salaire non
											renseigné
										</p>
									) : null}
								</div>
								<div className="shrink-0 text-right">
									<p className="font-semibold tabular-nums">
										{formatMinutes(row.planned_minutes)}
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
	projects,
	organizationSlug,
}: {
	title: string
	icon: React.ReactNode
	projects: ProjectProfitability[]
	organizationSlug: string
}) {
	return (
		<SectionCard>
			<SectionHeader title={title} />
			{projects.length === 0 ? (
				<p className="p-5 text-sm text-muted-foreground">
					Aucun projet n'a de marge calculable sur cette période.
				</p>
			) : (
				<ul className="divide-y">
					{projects.map((project) => (
						<li
							key={project.project_id}
							className="flex items-center justify-between gap-3 px-5 py-3"
						>
							<div className="flex min-w-0 items-center gap-2">
								{icon}
								<ProjectTitleLink
									project={project}
									organizationSlug={organizationSlug}
									className="truncate text-sm font-medium"
								/>
							</div>
							<div className="shrink-0 text-right">
								<p className="font-semibold tabular-nums">
									{project.margin_cents === null ||
									project.margin_cents === undefined
										? '—'
										: formatCents(project.margin_cents)}
								</p>
								<p className="text-xs text-muted-foreground">
									{formatMarginRate(project)}
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
 * A project name that opens its own page.
 *
 * Unlike the old version of this screen, which could only land a reader on the
 * planning task list because a chantier was not an addressable thing, a project
 * now has a page of its own.
 */
function ProjectTitleLink({
	project,
	organizationSlug,
	className,
}: {
	project: ProjectProfitability
	organizationSlug: string
	className?: string
}) {
	const expenses = expensesNote(project)

	return (
		<Link
			to={buildOrgPath(organizationSlug, '/planning/projects')}
			search={{ projectId: project.project_id }}
			title={expenses ? `Ouvrir le projet : ${expenses}` : 'Ouvrir le projet'}
			className={cn(
				'group inline-flex min-w-0 items-center gap-1 hover:underline',
				className,
			)}
		>
			<span className="truncate">{project.name}</span>
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

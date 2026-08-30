import { Link } from '@tanstack/react-router'
import { Skeleton } from '#/components/ui/skeleton'
import { buildOrgPath } from '#/modules/org-path'

export interface TodayTaskRow {
	id: string
	/** `"08:00–11:30"`, already formatted — see `today-planning-feature.tsx`. */
	timeWindow: string
	title: string
	/** The customer/context the task is on, when known — `null` for an
	 * internal task with neither. */
	subtitle: string | null
}

export interface TodayPlanningUIProps {
	organizationSlug: string
	entries: TodayTaskRow[]
	isLoading: boolean
	error: string | null
}

/**
 * Today's team-wide agenda, homepage-sized — a handful of rows, not the full
 * calendar. Distinct from `MyTasksTodayFeature` (the field pointage card
 * lower on this page): that one is the caller's own tasks with clock-in/out,
 * this one is what the whole team has on today, read-only.
 */
export function TodayPlanningUI({
	organizationSlug,
	entries,
	isLoading,
	error,
}: TodayPlanningUIProps) {
	return (
		<div className="flex flex-col rounded-xl bg-card p-5 shadow-sm">
			<div className="mb-1 flex items-baseline justify-between gap-4">
				<span className="island-kicker">Aujourd’hui</span>
				<Link
					to={buildOrgPath(organizationSlug, '/planning/calendar')}
					className="text-xs font-semibold text-brand-muted"
				>
					Voir le planning →
				</Link>
			</div>

			{isLoading ? (
				<div className="flex flex-col gap-3 py-3" aria-busy="true">
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-5/6" />
					<Skeleton className="h-4 w-2/3" />
				</div>
			) : error ? (
				<p className="py-3 text-sm text-destructive">{error}</p>
			) : entries.length === 0 ? (
				<p className="py-3 text-sm text-muted-foreground">
					Rien de planifié aujourd’hui.
				</p>
			) : (
				<ul>
					{entries.map((entry) => (
						<li
							key={entry.id}
							className="flex items-start gap-3 border-t py-3 first:border-t-0"
						>
							<span className="w-[4.6rem] shrink-0 pt-px text-sm font-semibold text-brand-muted tabular-nums">
								{entry.timeWindow}
							</span>
							<div className="min-w-0">
								<p className="truncate text-sm font-medium text-foreground">
									{entry.title}
								</p>
								{entry.subtitle ? (
									<p className="truncate text-xs text-muted-foreground">
										{entry.subtitle}
									</p>
								) : null}
							</div>
						</li>
					))}
				</ul>
			)}
		</div>
	)
}

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
		<section
			aria-label="Aujourd’hui"
			className="overflow-hidden rounded-xl border bg-card"
		>
			<header className="flex items-center justify-between border-b px-4 py-2.5">
				<h3 className="font-medium text-[13px]">Aujourd’hui</h3>
				<Link
					to={buildOrgPath(organizationSlug, '/planning/calendar')}
					className="text-[12px] text-primary outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Voir le planning →
				</Link>
			</header>

			{isLoading ? (
				<div className="flex flex-col gap-3 px-4 py-3" aria-busy="true">
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-5/6" />
					<Skeleton className="h-4 w-2/3" />
				</div>
			) : error ? (
				<p className="px-4 py-6 text-center text-[13px] text-destructive">
					{error}
				</p>
			) : entries.length === 0 ? (
				<p className="px-4 py-6 text-center text-[13px] text-muted-foreground">
					Rien de planifié aujourd’hui.
				</p>
			) : (
				<ul className="divide-y">
					{entries.map((entry) => (
						<li key={entry.id} className="flex items-start gap-3 px-4 py-3">
							<span className="tabular w-[4.6rem] shrink-0 pt-px font-medium text-[12.5px] text-primary">
								{entry.timeWindow}
							</span>
							<div className="min-w-0">
								<p className="truncate font-medium text-[13.5px]">
									{entry.title}
								</p>
								{entry.subtitle ? (
									<p className="truncate text-[12.5px] text-muted-foreground">
										{entry.subtitle}
									</p>
								) : null}
							</div>
						</li>
					))}
				</ul>
			)}
		</section>
	)
}

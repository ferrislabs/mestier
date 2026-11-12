import type { Schemas } from '#/api/api.client'
import { Skeleton } from '#/components/ui/skeleton'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { formatAdminNumber } from '#/pages/admin-layout/lib/admin-format'
import {
	type Bar,
	sortedAdoption,
	weeklyBars,
} from '#/pages/admin-stats/lib/stats'

interface AdminStatsProps {
	stats: Schemas.PlatformStatsResponse | undefined
	loading: boolean
	error: string | null
}

export function AdminStats({ stats, loading, error }: AdminStatsProps) {
	return (
		<div className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-6">
			<div>
				<h1 className="font-semibold text-xl">Vue d’ensemble</h1>
				<p className="text-muted-foreground text-sm">
					L’activité de la plateforme, toutes organisations confondues.
				</p>
			</div>
			{error ? (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			) : null}
			{loading || !stats ? (
				<Skeleton className="h-64 w-full" />
			) : (
				<>
					<div className="grid grid-cols-2 gap-3 md:grid-cols-3">
						<Tile label="Organisations" total={stats.active_organizations} />
						<Tile label="Utilisateurs" total={stats.active_users} />
						<Tile
							label="Devis"
							total={stats.quotes.total}
							recent={stats.quotes.last_30_days}
						/>
						<Tile
							label="Factures"
							total={stats.invoices.total}
							recent={stats.invoices.last_30_days}
						/>
						<Tile
							label="Projets"
							total={stats.projects.total}
							recent={stats.projects.last_30_days}
						/>
						<Tile
							label="Tâches"
							total={stats.tasks.total}
							recent={stats.tasks.last_30_days}
						/>
					</div>
					<div className="grid gap-3 md:grid-cols-2">
						<WeeklyChart
							title="Inscriptions par semaine"
							bars={weeklyBars(stats.signups_per_week)}
						/>
						<WeeklyChart
							title="Organisations créées par semaine"
							bars={weeklyBars(stats.organizations_per_week)}
						/>
					</div>
					<section className="flex flex-col gap-2">
						<h2 className="font-medium text-sm">Organisations par flag</h2>
						<div className="overflow-hidden rounded-lg border bg-card">
							<Table>
								<TableHeader>
									<TableRow className="hover:bg-transparent">
										<TableHead className="pl-4">Flag</TableHead>
										<TableHead className="pr-4 text-right">
											Organisations
										</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{sortedAdoption(stats.features).map((flag) => (
										<TableRow key={flag.key}>
											<TableCell className="pl-4">
												<span className="font-medium">{flag.label}</span>{' '}
												<span className="text-muted-foreground text-xs">
													{flag.key}
												</span>
											</TableCell>
											<TableCell className="tabular pr-4 text-right">
												{formatAdminNumber(flag.organizations)}
											</TableCell>
										</TableRow>
									))}
								</TableBody>
							</Table>
						</div>
					</section>
				</>
			)}
		</div>
	)
}

interface TileProps {
	label: string
	total: number
	recent?: number
}

function Tile({ label, total, recent }: TileProps) {
	return (
		<div className="rounded-lg border bg-card p-4">
			<p className="text-muted-foreground text-xs">{label}</p>
			<p className="tabular font-semibold text-2xl">
				{formatAdminNumber(total)}
			</p>
			{recent !== undefined ? (
				<p className="tabular text-muted-foreground text-xs">
					{formatAdminNumber(recent)} sur 30 j
				</p>
			) : null}
		</div>
	)
}

interface WeeklyChartProps {
	title: string
	bars: Bar[]
}

function WeeklyChart({ title, bars }: WeeklyChartProps) {
	return (
		<figure className="rounded-lg border bg-card p-4">
			<figcaption className="mb-3 font-medium text-sm">{title}</figcaption>
			<ol className="flex h-28 items-end gap-1">
				{bars.map((bar) => (
					<li
						key={bar.label}
						title={`${bar.label} : ${bar.count}`}
						className="flex h-full flex-1 flex-col justify-end"
					>
						<span
							className="block w-full min-h-px rounded-t-sm bg-primary"
							style={{ height: `${bar.percent}%` }}
						/>
					</li>
				))}
			</ol>
			<div className="mt-1 flex justify-between text-[10px] text-muted-foreground">
				<span>{bars[0]?.label}</span>
				<span>{bars.at(-1)?.label}</span>
			</div>
		</figure>
	)
}

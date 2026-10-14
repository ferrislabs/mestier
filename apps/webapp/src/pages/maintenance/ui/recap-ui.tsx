import {
	AlertCircle,
	CheckCircle2,
	ChevronLeft,
	ChevronRight,
	FileDown,
	ListChecks,
	Loader2,
	RefreshCw,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type {
	MaintenanceRecapGroup,
	MaintenanceRecapLine,
} from '#/hooks/use-maintenance-recap'
import {
	dueBadge,
	formatShortDate,
	hoursLabel,
	lineIdsOf,
	lineSiteLabel,
	townLabel,
} from '#/pages/maintenance/lib/recap-lines'

export interface RecapUIProps {
	organizationName: string
	/** Already worded: `S43 2026 · du 19 oct. au 25 oct.` */
	weekLabel: string
	groups: MaintenanceRecapGroup[]
	selected: ReadonlySet<string>
	isLoading: boolean
	/** The recap could not be read. */
	loadError: string | null
	onRetry: () => void
	/** Planning failed; the lines are still there. */
	planError: string | null
	/** What the last planning did, until the next one. */
	notice: string | null
	pdfError: string | null
	isPlanning: boolean
	isDownloadingPdf: boolean
	onPreviousWeek: () => void
	onNextWeek: () => void
	onToggleLine: (lineId: string) => void
	onToggleAll: () => void
	onPlan: () => void
	onDownloadPdf: () => void
}

/**
 * What the manager reads on Thursday: the lines due next week and the ones
 * left over, by town, with a box to tick each one to plan. Nothing fetches
 * here.
 */
export function RecapUI({
	organizationName,
	weekLabel,
	groups,
	selected,
	isLoading,
	loadError,
	onRetry,
	planError,
	notice,
	pdfError,
	isPlanning,
	isDownloadingPdf,
	onPreviousWeek,
	onNextWeek,
	onToggleLine,
	onToggleAll,
	onPlan,
	onDownloadPdf,
}: RecapUIProps) {
	const lineCount = lineIdsOf(groups).length
	const allSelected = lineCount > 0 && selected.size === lineCount
	const someSelected = selected.size > 0 && !allSelected

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Récap de la semaine"
				description="Les prestations à faire dans la semaine, et celles restées en retard, par commune."
				actions={
					<div className="flex flex-wrap items-center gap-2">
						<Button
							type="button"
							variant="outline"
							disabled={isDownloadingPdf || Boolean(loadError)}
							onClick={onDownloadPdf}
						>
							{isDownloadingPdf ? (
								<Loader2 className="animate-spin" />
							) : (
								<FileDown />
							)}
							PDF
						</Button>
						<Button
							type="button"
							disabled={selected.size === 0 || isPlanning}
							onClick={onPlan}
						>
							{isPlanning ? (
								<Loader2 className="animate-spin" />
							) : (
								<ListChecks />
							)}
							{selected.size > 0 ? `Planifier (${selected.size})` : 'Planifier'}
						</Button>
					</div>
				}
			/>

			<div className="flex items-center gap-2">
				<Button
					type="button"
					variant="outline"
					size="icon"
					aria-label="Semaine précédente"
					onClick={onPreviousWeek}
				>
					<ChevronLeft />
				</Button>
				<p className="min-w-64 text-center text-sm font-medium">{weekLabel}</p>
				<Button
					type="button"
					variant="outline"
					size="icon"
					aria-label="Semaine suivante"
					onClick={onNextWeek}
				>
					<ChevronRight />
				</Button>
			</div>

			{planError || pdfError ? (
				<SectionCard
					role="alert"
					className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive"
				>
					<AlertCircle className="size-5 shrink-0" />
					<p className="text-sm font-medium">{planError ?? pdfError}</p>
				</SectionCard>
			) : null}

			{notice ? (
				<SectionCard
					role="status"
					className="flex items-center gap-3 border-success/30 bg-success-soft p-5 text-success"
				>
					<CheckCircle2 className="size-5 shrink-0" />
					<p className="text-sm font-medium">{notice}</p>
				</SectionCard>
			) : null}

			{loadError ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{loadError}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						<RefreshCw />
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			<SectionCard>
				<SectionHeader
					title={`À planifier (${lineCount})`}
					description="Cochez les prestations, puis « Planifier » : une tâche sans date est créée pour chacune dans le tableau."
					actions={
						lineCount > 0 ? (
							<div className="flex items-center gap-2 text-sm">
								<Checkbox
									id="recap-select-all"
									checked={
										allSelected ? true : someSelected ? 'indeterminate' : false
									}
									onCheckedChange={onToggleAll}
								/>
								<label htmlFor="recap-select-all">Tout sélectionner</label>
							</div>
						) : null
					}
				/>

				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : lineCount === 0 && !loadError ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<ListChecks className="size-5" />
						</div>
						<p className="font-medium">Rien à planifier cette semaine.</p>
					</div>
				) : (
					groups.map((group) => (
						<div key={group.town ?? ''} className="border-b last:border-b-0">
							<h3 className="bg-muted/40 px-5 py-2 text-xs font-semibold uppercase text-muted-foreground">
								{townLabel(group.town)} ({group.lines.length})
							</h3>
							<ul className="divide-y">
								{group.lines.map((line) => (
									<RecapRow
										key={line.line_id}
										line={line}
										checked={selected.has(line.line_id)}
										onToggle={() => onToggleLine(line.line_id)}
									/>
								))}
							</ul>
						</div>
					))
				)}
			</SectionCard>
		</PageShell>
	)
}

function RecapRow({
	line,
	checked,
	onToggle,
}: {
	line: MaintenanceRecapLine
	checked: boolean
	onToggle: () => void
}) {
	const badge = dueBadge(line.due.status)
	const site = lineSiteLabel(line.site)

	return (
		<li className="flex items-start gap-3 px-5 py-4">
			<Checkbox
				className="mt-1"
				aria-label={`Sélectionner ${line.label}, ${site}`}
				checked={checked}
				onCheckedChange={onToggle}
			/>
			<div className="min-w-0 flex-1">
				<div className="flex flex-wrap items-center gap-2">
					<p className="text-sm font-medium">{line.label}</p>
					<StatusBadge tone={badge.tone}>{badge.label}</StatusBadge>
				</div>
				<p className="mt-0.5 text-sm text-muted-foreground">
					{site}
					{line.reference ? ` · ${line.reference}` : null}
				</p>
				{line.site.contact ? (
					<p className="mt-0.5 text-xs text-muted-foreground">
						{line.site.contact.name}
						{line.site.contact.phone ? ` · ${line.site.contact.phone}` : null}
					</p>
				) : null}
			</div>
			<dl className="grid shrink-0 grid-cols-3 gap-x-6 text-right text-xs text-muted-foreground">
				<div>
					<dt>Dernière</dt>
					<dd className="mt-0.5 text-sm text-foreground">
						{line.last_done_on ? formatShortDate(line.last_done_on) : 'Jamais'}
					</dd>
				</div>
				<div>
					<dt>Échéance</dt>
					<dd className="mt-0.5 text-sm text-foreground">
						{formatShortDate(line.due.on)}
					</dd>
				</div>
				<div>
					<dt>Prévu</dt>
					<dd className="mt-0.5 text-sm text-foreground">
						{hoursLabel(line.minutes_per_intervention)}
					</dd>
				</div>
			</dl>
		</li>
	)
}

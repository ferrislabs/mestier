import { AlertCircle, Loader2 } from 'lucide-react'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Grid } from '#/pages/maintenance/types'
import { GridLegend } from '#/pages/maintenance/ui/grid-legend'
import { MaintenanceGrid } from '#/pages/maintenance/ui/maintenance-grid'
import { SeasonPicker } from '#/pages/maintenance/ui/season-picker'
import {
	VisitEntryDialog,
	type VisitEntryDialogProps,
	type VisitEntryTarget,
} from '#/pages/maintenance/ui/visit-entry-dialog'

export interface GridUiProps {
	organizationName: string
	season: number
	grid: Grid | undefined
	currentWeek: number | null
	isLoading: boolean
	error: string | null
	entry: Omit<VisitEntryDialogProps, 'target'> & {
		target: VisitEntryTarget | null
	}
	onSeasonChange: (season: number) => void
	onCellClick: (target: VisitEntryTarget) => void
}

export function GridUi({
	organizationName,
	season,
	grid,
	currentWeek,
	isLoading,
	error,
	entry,
	onSeasonChange,
	onCellClick,
}: GridUiProps) {
	return (
		<PageShell className="max-w-none">
			<PageHeader
				eyebrow={organizationName}
				title="Grille d'entretien"
				description="Une ligne par prestation, une colonne par semaine. Cliquez une case pour saisir le jour de l'intervention."
				actions={<SeasonPicker season={season} onChange={onSeasonChange} />}
			/>

			<GridLegend />

			{error ? (
				<div
					role="alert"
					className="flex items-center gap-2 border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive"
				>
					<AlertCircle className="size-4" />
					{error}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center gap-2 text-sm text-muted-foreground">
					<Loader2 className="size-4 animate-spin" />
					Chargement de la grille
				</div>
			) : null}

			{grid && grid.contracts.length === 0 ? (
				<p className="border bg-card p-6 text-sm text-muted-foreground">
					Aucun contrat pour la saison {season}.
				</p>
			) : null}

			{grid && grid.contracts.length > 0 ? (
				<MaintenanceGrid
					grid={grid}
					currentWeek={currentWeek}
					onCellClick={onCellClick}
				/>
			) : null}

			<VisitEntryDialog {...entry} />
		</PageShell>
	)
}

import { AlertCircle, Loader2 } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
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
				title="Grille d'entretien"
				description="Une ligne par prestation, une colonne par semaine. Cliquez une case pour saisir le jour de l'intervention."
				actions={<SeasonPicker season={season} onChange={onSeasonChange} />}
			/>

			<GridLegend />

			{error ? (
				<SectionCard
					role="alert"
					className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-4 text-destructive"
				>
					<AlertCircle className="size-4 shrink-0" />
					<p className="text-[13px] font-medium">{error}</p>
				</SectionCard>
			) : null}

			{isLoading ? (
				<div className="flex items-center gap-2 text-[13px] text-muted-foreground">
					<Loader2 className="size-4 animate-spin" />
					Chargement de la grille
				</div>
			) : null}

			{grid && grid.contracts.length === 0 ? (
				<EmptyState
					title={`Aucun contrat pour la saison ${season}.`}
					hint="Créez un contrat dans la rubrique Contrats pour remplir la grille."
				/>
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

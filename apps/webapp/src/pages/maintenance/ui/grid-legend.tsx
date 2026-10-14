import { cn } from '#/lib/utils'
import type { CellState } from '#/pages/maintenance/lib/grid-model'
import { CELL_STYLES } from '#/pages/maintenance/ui/cell-styles'

const ENTRIES: { state: CellState; label: string; hint: string }[] = [
	{ state: 'validated', label: 'Validée', hint: 'Intervention faite' },
	{ state: 'proposed', label: 'Proposée', hint: 'À valider' },
	{ state: 'planned', label: 'Planifiée', hint: 'Tâche ouverte' },
	{ state: 'due', label: 'Échéance', hint: 'Prochain passage' },
	{ state: 'overdue', label: 'En retard', hint: 'Échéance dépassée' },
]

export function GridLegend() {
	return (
		<ul
			aria-label="Légende"
			className="flex flex-wrap items-center gap-x-5 gap-y-2"
		>
			{ENTRIES.map((entry) => (
				<li
					key={entry.state}
					title={entry.hint}
					className="flex items-center gap-2 text-sm text-muted-foreground"
				>
					<span
						aria-hidden="true"
						className={cn(
							'inline-flex size-6 items-center justify-center text-xs',
							CELL_STYLES[entry.state],
						)}
					>
						12
					</span>
					<span>{entry.label}</span>
				</li>
			))}
		</ul>
	)
}

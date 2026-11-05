import type { CellState } from '#/pages/maintenance/lib/grid-model'

/**
 * The colours of the sheet. Colour is never the only signal: every cell also
 * carries its state in its accessible name, and the legend spells them out.
 */
export const CELL_STYLES: Record<CellState, string> = {
	validated: 'bg-success text-card',
	proposed: 'border border-dashed border-success bg-success-soft text-success',
	planned: 'bg-state-progress/15 text-state-progress',
	due: 'border border-warning bg-warning-soft text-warning italic',
	overdue: 'bg-destructive font-semibold text-destructive-foreground',
	empty: '',
	'out-of-season':
		'bg-[repeating-linear-gradient(135deg,transparent,transparent_3px,var(--border)_3px,var(--border)_4px)]',
}

export const STATE_LABELS: Record<CellState, string> = {
	validated: 'validée',
	proposed: 'proposée',
	planned: 'planifiée',
	due: 'échéance',
	overdue: 'en retard',
	empty: 'vide',
	'out-of-season': 'hors saison',
}

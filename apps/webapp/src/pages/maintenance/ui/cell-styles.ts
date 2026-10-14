import type { CellState } from '#/pages/maintenance/lib/grid-model'

/**
 * The colours of the sheet. Colour is never the only signal: every cell also
 * carries its state in its accessible name, and the legend spells them out.
 */
export const CELL_STYLES: Record<CellState, string> = {
	validated:
		'bg-emerald-600 text-white dark:bg-emerald-500 dark:text-emerald-950',
	proposed:
		'border border-dashed border-emerald-600 bg-emerald-50 text-emerald-900 dark:border-emerald-400 dark:bg-emerald-950 dark:text-emerald-100',
	planned: 'bg-sky-100 text-sky-900 dark:bg-sky-950 dark:text-sky-100',
	due: 'border border-amber-500 bg-amber-50 text-amber-900 italic dark:bg-amber-950 dark:text-amber-100',
	overdue:
		'bg-red-600 text-white font-semibold dark:bg-red-500 dark:text-red-950',
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

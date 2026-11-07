const RULES: readonly (readonly [legacy: string, current: string])[] = [
	['customers/pipeline', 'pipeline'],
	['customers', 'clients'],
	['quotes/new', 'devis/nouveau'],
	['quotes', 'devis'],
	['invoices/new', 'factures/nouvelle'],
	['invoices', 'factures'],
	['credit-notes', 'avoirs'],
	['treasury', 'tresorerie'],
	['catalog', 'catalogue'],
	['settings', 'reglages'],
]

const LEGACY_HANDOVER = /\/handover$/

/** Current path of a pre-rework CRM URL, or `null` when `splat` is not one. */
export function legacyCrmPath(splat: string): string | null {
	for (const [legacy, current] of RULES) {
		if (splat === legacy || splat.startsWith(`${legacy}/`)) {
			const rest = splat
				.slice(legacy.length)
				.replace(LEGACY_HANDOVER, '/passation')
			return `/crm/${current}${rest}`
		}
	}
	return null
}

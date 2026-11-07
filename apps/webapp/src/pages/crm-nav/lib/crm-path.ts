export const ENTRY = {
	clients: 'clients',
	pipeline: 'pipeline',
	quotes: 'quotes',
	invoices: 'invoices',
	creditNotes: 'credit-notes',
	ledger: 'ledger',
	receipts: 'receipts',
	expenses: 'expenses',
	stripe: 'stripe',
	catalog: 'catalog',
	suppliers: 'suppliers',
	settings: 'settings',
} as const

const SECTIONS: readonly (readonly [string, string])[] = [
	['/crm/clients', ENTRY.clients],
	['/crm/pipeline', ENTRY.pipeline],
	['/crm/devis', ENTRY.quotes],
	['/crm/factures', ENTRY.invoices],
	['/crm/avoirs', ENTRY.creditNotes],
	['/crm/stripe', ENTRY.stripe],
	['/crm/catalogue', ENTRY.catalog],
	['/crm/fournisseurs', ENTRY.suppliers],
	['/crm/reglages', ENTRY.settings],
]

function isUnder(modulePath: string, prefix: string): boolean {
	return modulePath === prefix || modulePath.startsWith(`${prefix}/`)
}

export function activeSidebarEntries(
	modulePath: string,
	search: Record<string, unknown>,
): Set<string> {
	if (isUnder(modulePath, '/crm/tresorerie')) {
		if (search.type === 'recettes') return new Set([ENTRY.receipts])
		if (search.type === 'depenses') return new Set([ENTRY.expenses])
		return new Set([ENTRY.ledger])
	}
	const section = SECTIONS.find(([prefix]) => isUnder(modulePath, prefix))
	return section ? new Set([section[1]]) : new Set()
}

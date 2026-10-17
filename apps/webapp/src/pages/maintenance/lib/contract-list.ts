import { describeCadence } from '#/pages/maintenance/lib/cadence-form'
import type { MaintenanceContract } from '#/pages/maintenance/types'

/** "GAUME - Jardin", or whichever half is known. */
export function placeLabel(
	customerName: string | null,
	siteLabel: string | null,
): string {
	if (customerName && siteLabel && siteLabel !== customerName) {
		return `${customerName} - ${siteLabel}`
	}

	return customerName ?? siteLabel ?? 'Client inconnu'
}

/** "Tonte (Tous les 14 jours)", one per line. */
export function lineSummaries(contract: MaintenanceContract): string[] {
	return contract.lines.map(
		(line) => `${line.label} (${describeCadence(line.cadence)})`,
	)
}

/**
 * The contracts a person still has to review (`DRAFT`), apart from the rest.
 * An archived draft stays in the main list: it is out of the way already, and
 * there is nothing left to activate.
 */
export function splitDrafts<T extends { contract: MaintenanceContract }>(
	rows: T[],
): { drafts: T[]; active: T[] } {
	const drafts: T[] = []
	const active: T[] = []
	for (const row of rows) {
		const toReview =
			row.contract.status === 'DRAFT' && row.contract.archived_at == null
		;(toReview ? drafts : active).push(row)
	}

	return { drafts, active }
}

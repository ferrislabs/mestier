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

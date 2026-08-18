import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'

/**
 * How each billable unit is written for a French reader.
 *
 * Kept in one place because the catalogue, the quote form and the quote
 * summary all name the same units, and three copies of the map is how adding
 * a unit last time broke two screens that nobody thought to open.
 */
const UNIT_LABELS: Record<ServiceRateUnit, string> = {
	FLAT_RATE: 'forfait',
	HOUR: 'heure',
	DAY: 'jour',
	UNIT: 'unité',
	ML: 'ml',
	M2: 'm²',
	M3: 'm³',
	KG: 'kg',
	TONNE: 't',
	LITRE: 'L',
}

export function formatUnit(unit: ServiceRateUnit): string {
	return UNIT_LABELS[unit] ?? unit
}

/**
 * The spelled-out form, for a label that has room: `mètre carré` over `m²`.
 */
const UNIT_LABELS_LONG: Record<ServiceRateUnit, string> = {
	FLAT_RATE: 'forfait',
	HOUR: 'heure',
	DAY: 'jour',
	UNIT: 'unité',
	ML: 'mètre linéaire',
	M2: 'mètre carré',
	M3: 'mètre cube',
	KG: 'kilogramme',
	TONNE: 'tonne',
	LITRE: 'litre',
}

export function formatUnitLong(unit: ServiceRateUnit): string {
	return UNIT_LABELS_LONG[unit] ?? formatUnit(unit)
}

/** A price expressed per unit, as a catalogue reads it: `€/m²`. */
export function formatPricePerUnit(unit: ServiceRateUnit): string {
	return `€/${formatUnit(unit)}`
}

/**
 * The unit picker, grouped so ten options stay scannable. Ordered the way a
 * quote is written: how the job is billed first, then what it is measured in.
 */
export const UNIT_GROUPS: {
	label: string
	units: ServiceRateUnit[]
}[] = [
	{ label: 'Facturation', units: ['FLAT_RATE', 'HOUR', 'DAY', 'UNIT'] },
	{ label: 'Longueurs et surfaces', units: ['ML', 'M2', 'M3'] },
	{ label: 'Masse et volume', units: ['KG', 'TONNE', 'LITRE'] },
]

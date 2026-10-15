import type { MaintenanceGrid } from '#/hooks/use-maintenance-validation'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	formatShortDate,
	lineSiteLabel,
} from '#/pages/maintenance/lib/recap-lines'

export interface VisitLine {
	label: string
	site: string
	reference: string | null
}

/**
 * The seasons to read to name the lines of a queue. A visit is dated inside
 * its contract's season, so the year of the day is the season to ask for.
 */
export function seasonsOf(visits: { done_on: string }[]): number[] {
	const years = new Set(
		visits.map((visit) => Number(visit.done_on.slice(0, 4))),
	)
	return [...years].sort((a, b) => a - b)
}

export function visitLineIndex(
	grids: (MaintenanceGrid | undefined)[],
): Map<string, VisitLine> {
	const index = new Map<string, VisitLine>()
	for (const grid of grids) {
		for (const contract of grid?.contracts ?? []) {
			for (const line of contract.lines) {
				index.set(line.id, {
					label: line.label,
					site: lineSiteLabel(contract.site),
					reference: contract.reference ?? null,
				})
			}
		}
	}
	return index
}

export function describeVisit(
	lineId: string,
	index: ReadonlyMap<string, VisitLine>,
): { title: string; subtitle: string } {
	const line = index.get(lineId)
	if (!line) {
		return {
			title: 'Prestation introuvable',
			subtitle: 'Son contrat est peut-être archivé.',
		}
	}
	return {
		title: line.label,
		subtitle: line.reference ? `${line.site} · ${line.reference}` : line.site,
	}
}

export function interventionActionError(error: unknown): string | null {
	if (error == null) return null
	const status =
		typeof error === 'object' ? (error as { status?: unknown }).status : null
	if (status === 409) {
		return 'Cette visite a déjà été traitée. La file est actualisée.'
	}
	if (status === 404) {
		return 'Cette visite n’existe plus. La file est actualisée.'
	}
	return mutationErrorMessage(error)
}

export interface QueueRow {
	id: string
	title: string
	subtitle: string
	proposedBy: string
	/** `14 oct.` */
	doneOn: string
}

export function buildQueueRows(
	visits: {
		id: string
		line_id: string
		done_on: string
		proposed_by: string
	}[],
	index: ReadonlyMap<string, VisitLine>,
	memberName: (memberId: string) => string,
): QueueRow[] {
	return visits.map((visit) => ({
		id: visit.id,
		...describeVisit(visit.line_id, index),
		proposedBy: memberName(visit.proposed_by),
		doneOn: formatShortDate(visit.done_on),
	}))
}

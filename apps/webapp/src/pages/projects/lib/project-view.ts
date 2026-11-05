import type { Doe } from '#/hooks/use-doe'
import { formatDate, formatShortDate, yearOf } from '#/pages/projects/lib/dates'
import type { MissionStats } from '#/pages/projects/lib/missions'
import type { IsoDate } from '#/pages/projects/lib/model'
import { countLabel } from '#/pages/projects/lib/text'

export interface DoeSummary {
	revision: string
	enabledSections: number
	supplies: number
	photos: number
	plans: number
}

export function doeSummary(doe: Doe): DoeSummary {
	let supplies = 0
	let photos = 0
	let plans = 0
	for (const section of doe.sections) {
		if (!section.enabled) continue
		if (section.content.kind === 'supplies')
			supplies += section.content.supplies.length
		if (section.content.kind === 'photos')
			photos += section.content.photos.length
		if (section.content.kind === 'plans') plans += section.content.plans.length
	}
	return {
		revision: doe.revision,
		enabledSections: doe.sections.filter((section) => section.enabled).length,
		supplies,
		photos,
		plans,
	}
}

export function periodLabel(
	start: IsoDate | null,
	end: IsoDate | null,
): string | null {
	if (start && end)
		return `${yearOf(start) === yearOf(end) ? formatShortDate(start) : formatDate(start)} → ${formatDate(end)}`
	if (start) return `Dès le ${formatDate(start)}`
	if (end) return `Jusqu’au ${formatDate(end)}`
	return null
}

export function missionsRatio(stats: MissionStats): string {
	return `${stats.done} / ${stats.total}`
}

export function doeTileLabel(doe: Pick<DoeSummary, 'revision'> | null): string {
	return doe ? `Indice ${doe.revision}` : 'Pas de DOE'
}

export function doeSummaryParts(summary: DoeSummary): string[] {
	return [
		countLabel(summary.enabledSections, 'thème actif', 'thèmes actifs'),
		countLabel(summary.supplies, 'fourniture', 'fournitures'),
		countLabel(summary.photos, 'photo', 'photos'),
		countLabel(summary.plans, 'plan', 'plans'),
	]
}

import type { Schemas } from '#/api/api.client'
import { weekLabel } from '#/pages/maintenance/lib/weeks'
import type { Cadence } from '#/pages/maintenance/types'

export type CadenceKind = Cadence['kind']

export interface WindowDraft {
	from: string
	to: string
}

/**
 * The cadence picker keeps the fields of every kind, so flipping from "tous
 * les N jours" to "fenêtres" and back does not lose what was typed.
 */
export interface CadenceDraft {
	kind: CadenceKind
	everyDays: string
	windows: WindowDraft[]
}

export const MAX_INTERVAL_DAYS = 365
export const MAX_WINDOWS = 12
const MAX_ISO_WEEK = 53

export const CADENCE_KINDS: { kind: CadenceKind; label: string }[] = [
	{ kind: 'ROLLING', label: 'Tous les N jours' },
	{ kind: 'WINDOWS', label: 'Fenêtres de semaines' },
	{ kind: 'ON_DEMAND', label: 'À la demande' },
]

export function emptyCadenceDraft(): CadenceDraft {
	return { kind: 'ROLLING', everyDays: '14', windows: [] }
}

export function cadenceToDraft(cadence: Cadence): CadenceDraft {
	const draft = emptyCadenceDraft()
	switch (cadence.kind) {
		case 'ROLLING':
			return { ...draft, everyDays: String(cadence.every_days) }
		case 'WINDOWS':
			return {
				...draft,
				kind: 'WINDOWS',
				windows: cadence.windows.map((window) => ({
					from: String(window.from_week),
					to: String(window.to_week),
				})),
			}
		case 'ON_DEMAND':
			return { ...draft, kind: 'ON_DEMAND' }
	}
}

export interface SeasonWeeks {
	firstWeek: number
	lastWeek: number
}

function wholeNumber(value: string): number | null {
	const trimmed = value.trim()
	if (!/^\d+$/.test(trimmed)) return null

	return Number(trimmed)
}

function windowError(window: WindowDraft, season: SeasonWeeks): string | null {
	const from = wholeNumber(window.from)
	const to = wholeNumber(window.to)
	if (from === null || to === null || from < 1 || to < 1) {
		return 'Chaque fenêtre a une semaine de début et une de fin.'
	}
	if (from > MAX_ISO_WEEK || to > MAX_ISO_WEEK) {
		return `Les semaines vont de 1 à ${MAX_ISO_WEEK}.`
	}
	if (to < from) {
		return `La fenêtre ${weekLabel(from)} à ${weekLabel(to)} finit avant de commencer.`
	}
	if (from < season.firstWeek || to > season.lastWeek) {
		return `La fenêtre ${weekLabel(from)} à ${weekLabel(to)} sort de la saison (${weekLabel(season.firstWeek)} à ${weekLabel(season.lastWeek)}).`
	}

	return null
}

/**
 * The first reason a cadence would be refused, in the words of the form, or
 * `null`. It mirrors the backend's rules (1 to 365 days; 1 to 12 windows,
 * inside the season, none overlapping) so the person is told before the
 * round trip; the backend still has the last word.
 */
export function cadenceError(
	draft: CadenceDraft,
	season: SeasonWeeks,
): string | null {
	if (draft.kind === 'ON_DEMAND') return null

	if (draft.kind === 'ROLLING') {
		const days = wholeNumber(draft.everyDays)
		if (days === null || days < 1 || days > MAX_INTERVAL_DAYS) {
			return `L'intervalle va de 1 à ${MAX_INTERVAL_DAYS} jours.`
		}

		return null
	}

	if (draft.windows.length === 0) return 'Ajoutez au moins une fenêtre.'
	if (draft.windows.length > MAX_WINDOWS) {
		return `Une prestation a au plus ${MAX_WINDOWS} fenêtres.`
	}
	for (const window of draft.windows) {
		const error = windowError(window, season)
		if (error) return error
	}

	const ordered = draft.windows
		.map((window) => ({ from: Number(window.from), to: Number(window.to) }))
		.sort((a, b) => a.from - b.from)
	for (let index = 1; index < ordered.length; index += 1) {
		if (ordered[index].from <= ordered[index - 1].to) {
			return 'Deux fenêtres se chevauchent.'
		}
	}

	return null
}

/** Only call once {@link cadenceError} is `null`. */
export function toCadenceRequest(draft: CadenceDraft): Schemas.CadenceRequest {
	switch (draft.kind) {
		case 'ROLLING':
			return { kind: 'ROLLING', every_days: Number(draft.everyDays) }
		case 'WINDOWS':
			return {
				kind: 'WINDOWS',
				windows: draft.windows.map((window) => ({
					from_week: Number(window.from),
					to_week: Number(window.to),
				})),
			}
		case 'ON_DEMAND':
			return { kind: 'ON_DEMAND' }
	}
}

/** One line of text for a list or a tooltip: "Tous les 14 jours". */
export function describeCadence(cadence: Cadence): string {
	switch (cadence.kind) {
		case 'ROLLING':
			return cadence.every_days === 1
				? 'Tous les jours'
				: `Tous les ${cadence.every_days} jours`
		case 'WINDOWS':
			return cadence.windows
				.map((window) =>
					window.from_week === window.to_week
						? weekLabel(window.from_week)
						: `${weekLabel(window.from_week)}-${weekLabel(window.to_week)}`,
				)
				.join(', ')
		case 'ON_DEMAND':
			return 'À la demande'
	}
}

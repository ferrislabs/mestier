import type { Schemas } from '#/api/api.client'
import { mutationErrorMessage } from '#/lib/api-error'
import type { DayOverride, IsoDate, RhState } from '#/pages/hr/lib/rh/types'

export type DayOverrideRecord = Schemas.DayOverrideResponse
export type DayOverrideRequest = Schemas.PutDayOverrideRequest

export type OverridePeriod = {
	memberId: string | null
	from: IsoDate
	to: IsoDate
	records: readonly DayOverrideRecord[]
}

export function overrideOf(record: DayOverrideRecord): DayOverride {
	if (record.kind === 'OFF')
		return {
			memberId: record.member_id,
			date: record.date,
			kind: 'off',
			start: '',
			end: '',
			lunch: null,
			note: record.note,
		}
	return {
		memberId: record.member_id,
		date: record.date,
		kind: 'work',
		start: record.start ?? '',
		end: record.end ?? '',
		lunch: record.lunch
			? { start: record.lunch.start, end: record.lunch.end }
			: null,
		note: record.note,
	}
}

export function overrideRequestOf(draft: DayOverride): DayOverrideRequest {
	if (draft.kind === 'off') return { kind: 'OFF', note: draft.note.trim() }
	return {
		kind: 'WORK',
		start: draft.start,
		end: draft.end,
		lunch: draft.lunch
			? { start: draft.lunch.start, end: draft.lunch.end }
			: null,
		note: draft.note.trim(),
	}
}

export function periodKey(
	memberId: string | null,
	from: IsoDate,
	to: IsoDate,
): string {
	return `${memberId ?? '*'}|${from}|${to}`
}

export function mergeOverridePeriods(
	periods: readonly OverridePeriod[],
): DayOverrideRecord[] {
	return periods.reduce<DayOverrideRecord[]>(
		(merged, period) => [
			...merged.filter(
				(record) =>
					record.date < period.from ||
					record.date > period.to ||
					(period.memberId !== null && record.member_id !== period.memberId),
			),
			...period.records,
		],
		[],
	)
}

export function overlayOverrides(
	state: RhState,
	periods: readonly OverridePeriod[],
): RhState {
	return {
		...state,
		overrides: mergeOverridePeriods(periods).map(overrideOf),
	}
}

export type OverrideFailureField = 'day' | 'lunch' | 'kind'

export type OverrideFailure = {
	field: OverrideFailureField | null
	message: string
}

type ApiFailure = {
	status?: number
	data?: { message?: string; details?: { field?: string } }
}

export function overrideFailureOf(error: unknown): OverrideFailure {
	const failure = (
		typeof error === 'object' && error !== null ? error : {}
	) as ApiFailure
	if (failure.status === 404)
		return { field: null, message: 'Ce membre est introuvable.' }
	const path = failure.data?.details?.field ?? ''
	if (failure.status === 422) {
		if (path === 'lunch')
			return {
				field: 'lunch',
				message: 'La pause doit tenir dans la journée, la fin après le début.',
			}
		if (path === 'kind')
			return { field: 'kind', message: 'Un jour de repos n’a pas d’horaires.' }
		return {
			field: 'day',
			message: 'L’heure de fin doit suivre l’heure de début (HH:MM).',
		}
	}
	return {
		field: null,
		message: mutationErrorMessage(error) ?? 'Une erreur est survenue.',
	}
}

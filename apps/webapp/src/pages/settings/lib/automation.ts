import type { AutomationSettings } from '#/hooks/use-automation'

export interface SettingsFormValues {
	eventRetentionSeconds: string
	succeededRunRetentionSeconds: string
	/** Comma-separated seconds, e.g. "5, 30, 120, 600, 3600, 21600" — one
	 * entry per retry attempt, in order. Kept as plain seconds rather than a
	 * bespoke duration syntax: fewer ways to mistype, and
	 * `formatDurationSeconds` already gives a human-readable preview
	 * alongside the field. */
	retryScheduleSeconds: string
	/** Empty string means "never disable" (`null` on the wire) — distinct
	 * from `0`, which the backend's own bounds reject anyway. */
	disableTargetAfter: string
}

export function settingsToFormValues(
	settings: AutomationSettings,
): SettingsFormValues {
	return {
		eventRetentionSeconds: String(settings.event_retention_seconds),
		succeededRunRetentionSeconds: String(
			settings.succeeded_run_retention_seconds,
		),
		retryScheduleSeconds: settings.retry_schedule_seconds.join(', '),
		disableTargetAfter:
			settings.disable_target_after == null
				? ''
				: String(settings.disable_target_after),
	}
}

export interface ParsedSettings {
	ok: true
	body: AutomationSettings
}

export interface InvalidSettings {
	ok: false
	error: string
}

/**
 * Parses the form into the request body, or a single readable error.
 * Deliberately does not check the instance bounds itself — those are the
 * backend's own numbers (`SettingsBounds`, self-hosted vs. hosted), and the
 * acceptance criterion is that a value outside them is refused by the real
 * request with the backend's message, never silently clamped on this side.
 * This only catches what would otherwise crash the request: text where a
 * number belongs.
 */
export function parseSettingsForm(
	values: SettingsFormValues,
): ParsedSettings | InvalidSettings {
	const eventRetentionSeconds = parsePositiveInt(values.eventRetentionSeconds)
	if (eventRetentionSeconds === null) {
		return { ok: false, error: 'Rétention des événements : nombre requis' }
	}

	const succeededRunRetentionSeconds = parsePositiveInt(
		values.succeededRunRetentionSeconds,
	)
	if (succeededRunRetentionSeconds === null) {
		return { ok: false, error: 'Rétention des runs réussis : nombre requis' }
	}

	const retrySchedule = parseRetrySchedule(values.retryScheduleSeconds)
	if (retrySchedule === null) {
		return {
			ok: false,
			error:
				'Plan de nouvelles tentatives : liste de secondes séparées par des virgules (ex. 5, 30, 120)',
		}
	}

	let disableTargetAfter: number | null = null
	if (values.disableTargetAfter.trim() !== '') {
		disableTargetAfter = parsePositiveInt(values.disableTargetAfter)
		if (disableTargetAfter === null) {
			return {
				ok: false,
				error:
					'Seuil de désactivation : nombre requis, ou vide pour ne jamais désactiver',
			}
		}
	}

	return {
		ok: true,
		body: {
			event_retention_seconds: eventRetentionSeconds,
			succeeded_run_retention_seconds: succeededRunRetentionSeconds,
			retry_schedule_seconds: retrySchedule,
			disable_target_after: disableTargetAfter,
		},
	}
}

function parsePositiveInt(value: string): number | null {
	const trimmed = value.trim()
	if (trimmed === '') return null
	const parsed = Number.parseInt(trimmed, 10)
	if (!Number.isFinite(parsed) || parsed <= 0) return null
	return parsed
}

function parseRetrySchedule(value: string): number[] | null {
	const trimmed = value.trim()
	if (trimmed === '') return null

	const parts = trimmed.split(',').map((part) => part.trim())
	const seconds: number[] = []
	for (const part of parts) {
		const parsed = parsePositiveInt(part)
		if (parsed === null) return null
		seconds.push(parsed)
	}
	return seconds
}

/** "5s" / "2min" / "1h" / "3j" — the unit this screen displays retry
 * intervals and retention windows in, everywhere. */
export function formatDurationSeconds(seconds: number): string {
	if (seconds < 60) return `${seconds}s`
	if (seconds < 3600) return `${Math.round(seconds / 60)}min`
	if (seconds < 86_400) return `${Math.round(seconds / 3600)}h`
	return `${Math.round(seconds / 86_400)}j`
}

export function formatRetrySchedulePreview(seconds: number[]): string {
	if (seconds.length === 0) return '—'
	return seconds.map(formatDurationSeconds).join(' · ')
}

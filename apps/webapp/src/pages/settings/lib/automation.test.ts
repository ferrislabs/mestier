import { describe, expect, it } from 'vitest'
import type { AuthScheme } from '#/hooks/use-automation'
import {
	buildCredentialFormErrors,
	canReplay,
	formatDurationSeconds,
	formatRetrySchedulePreview,
	parseSettingsForm,
	settingsToFormValues,
} from '#/pages/settings/lib/automation'
import type { CredentialFormValues } from '#/pages/settings/types'

describe('settingsToFormValues / parseSettingsForm — round trip', () => {
	it('round-trips the defaults unchanged', () => {
		const settings = {
			event_retention_seconds: 7_776_000,
			succeeded_run_retention_seconds: 2_592_000,
			retry_schedule_seconds: [5, 30, 120, 600, 3600, 21_600],
			disable_target_after: 20,
		}

		const values = settingsToFormValues(settings)
		const result = parseSettingsForm(values)

		expect(result.ok).toBe(true)
		expect(result.ok && result.body).toEqual(settings)
	})

	it('renders a null threshold as an empty field, and parses it back to null', () => {
		const values = settingsToFormValues({
			event_retention_seconds: 100,
			succeeded_run_retention_seconds: 100,
			retry_schedule_seconds: [5],
			disable_target_after: null,
		})

		expect(values.disableTargetAfter).toBe('')

		const result = parseSettingsForm(values)
		expect(result.ok && result.body.disable_target_after).toBeNull()
	})
})

describe('parseSettingsForm — validation', () => {
	function validValues() {
		return settingsToFormValues({
			event_retention_seconds: 100,
			succeeded_run_retention_seconds: 100,
			retry_schedule_seconds: [5, 30],
			disable_target_after: 10,
		})
	}

	it('rejects a non-numeric event retention', () => {
		const result = parseSettingsForm({
			...validValues(),
			eventRetentionSeconds: 'abc',
		})

		expect(result.ok).toBe(false)
	})

	it('rejects a retry schedule that does not parse as a comma-separated list', () => {
		const result = parseSettingsForm({
			...validValues(),
			retryScheduleSeconds: '5, abc, 30',
		})

		expect(result.ok).toBe(false)
	})

	it('rejects an empty retry schedule', () => {
		const result = parseSettingsForm({
			...validValues(),
			retryScheduleSeconds: '',
		})

		expect(result.ok).toBe(false)
	})

	it('accepts a blank disable-after as "never disable"', () => {
		const result = parseSettingsForm({
			...validValues(),
			disableTargetAfter: '  ',
		})

		expect(result.ok).toBe(true)
		expect(result.ok && result.body.disable_target_after).toBeNull()
	})

	it('rejects a non-numeric disable-after', () => {
		const result = parseSettingsForm({
			...validValues(),
			disableTargetAfter: 'nope',
		})

		expect(result.ok).toBe(false)
	})

	// Deliberately does not test instance-bound rejection (e.g. a retry
	// below the floor): those bounds are the backend's own numbers and the
	// acceptance criterion is that they are enforced by the real request,
	// never guessed or duplicated on this side.
})

describe('formatDurationSeconds', () => {
	it('formats seconds, minutes, hours and days at their own scale', () => {
		expect(formatDurationSeconds(5)).toBe('5s')
		expect(formatDurationSeconds(120)).toBe('2min')
		expect(formatDurationSeconds(3600)).toBe('1h')
		expect(formatDurationSeconds(3 * 86_400)).toBe('3j')
	})
})

describe('formatRetrySchedulePreview', () => {
	it('joins every entry with a middle dot', () => {
		expect(formatRetrySchedulePreview([5, 30, 3600])).toBe('5s · 30s · 1h')
	})

	it('renders an em dash for an empty schedule', () => {
		expect(formatRetrySchedulePreview([])).toBe('—')
	})
})

describe('canReplay', () => {
	it('allows replay once a run has settled', () => {
		expect(canReplay('succeeded')).toBe(true)
		expect(canReplay('failed')).toBe(true)
		expect(canReplay('cancelled')).toBe(true)
	})

	it('refuses replay while a run is still pending or running', () => {
		expect(canReplay('pending')).toBe(false)
		expect(canReplay('running')).toBe(false)
	})
})

describe('buildCredentialFormErrors', () => {
	const bearerScheme: AuthScheme = {
		kind: 'bearer_token',
		label: 'Bearer token',
		fields: [
			{
				name: 'token',
				label: 'Token',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	}

	function values(overrides: Partial<CredentialFormValues> = {}) {
		return {
			kind: 'bearer_token',
			name: 'Ma clé',
			origin: 'supplied' as const,
			data: {},
			...overrides,
		}
	}

	it('requires a name', () => {
		const errors = buildCredentialFormErrors(
			values({ name: '' }),
			bearerScheme,
			'create',
		)
		expect(errors).toContain('Le nom est requis')
	})

	it('requires every scheme field on create', () => {
		const errors = buildCredentialFormErrors(values(), bearerScheme, 'create')
		expect(errors).toContain('Token est requis')
	})

	it('accepts a fully filled scheme on create', () => {
		const errors = buildCredentialFormErrors(
			values({ data: { token: 'abc' } }),
			bearerScheme,
			'create',
		)
		expect(errors).toEqual([])
	})

	it('allows every field blank on edit — rename only, data untouched', () => {
		const errors = buildCredentialFormErrors(values(), bearerScheme, 'edit')
		expect(errors).toEqual([])
	})

	it('requires the field once the user starts filling data in on edit', () => {
		const errors = buildCredentialFormErrors(
			values({ data: { token: '' } }),
			bearerScheme,
			'edit',
		)
		// An entry present but blank does not count as "started filling in" —
		// only a non-blank value does (see the implementation's `filledAny`).
		expect(errors).toEqual([])
	})

	it('skips scheme validation entirely for a generated credential', () => {
		const errors = buildCredentialFormErrors(
			values({ origin: 'generated' }),
			bearerScheme,
			'create',
		)
		expect(errors).toEqual([])
	})
})

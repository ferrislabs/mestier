import { describe, expect, it } from 'vitest'
import {
	formatDurationSeconds,
	formatRetrySchedulePreview,
	parseSettingsForm,
	settingsToFormValues,
} from '#/pages/settings/lib/automation'

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

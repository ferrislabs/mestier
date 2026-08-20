import { describe, expect, it } from 'vitest'
import { fieldErrorMessage } from '#/pages/field/field-errors'

describe('fieldErrorMessage', () => {
	it('maps each known backend conflict message to its French wording', () => {
		expect(
			fieldErrorMessage('Conflict: a time entry cannot end before it started'),
		).toBe("L'heure de fin doit être après l'heure de début.")
		expect(
			fieldErrorMessage(
				'Conflict: this stretch began on an earlier day and needs its end time declared',
			),
		).toBe(
			"Ce pointage a commencé un autre jour : indiquez l'heure de fin réelle.",
		)
		expect(
			fieldErrorMessage(
				'Conflict: this stretch is from today, so it is stopped rather than recovered',
			),
		).toBe(
			"Ce pointage date d'aujourd'hui : utilisez plutôt le bouton d'arrêt normal.",
		)
		expect(fieldErrorMessage('Conflict: time entry is already closed')).toBe(
			'Ce pointage est déjà clôturé.',
		)
	})

	/** The one message that carries a dynamic suffix — a task id. */
	it('matches the already-clocked-on message regardless of the task id it names', () => {
		expect(
			fieldErrorMessage(
				'Conflict: employee is already clocked on to task 3fae1c9e-1234-4c00-9c00-abcdefabcdef',
			),
		).toBe('Vous êtes déjà pointé sur un autre projet.')
	})

	it('falls back to a generic French message for anything unmapped', () => {
		expect(fieldErrorMessage('Conflict: something entirely unexpected')).toBe(
			'Une erreur est survenue, réessayez.',
		)
		expect(fieldErrorMessage('Resource not found')).toBe(
			'Une erreur est survenue, réessayez.',
		)
	})

	it('never lets the raw English message pass through unmapped', () => {
		const message = fieldErrorMessage('Conflict: time entry is already closed')
		expect(message).not.toMatch(/already closed/i)
	})
})

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

	it('maps a declared end in the future', () => {
		expect(
			fieldErrorMessage('Conflict: a time entry cannot end in the future'),
		).toBe("L'heure de fin ne peut pas être dans le futur.")
	})

	/** Also carries a dynamic task id suffix. */
	it('maps a declared stretch overlapping a job still running, regardless of the task id it names', () => {
		expect(
			fieldErrorMessage(
				'Conflict: declared stretch overlaps the job the employee is still clocked on to, task 3fae1c9e-1234-4c00-9c00-abcdefabcdef',
			),
		).toBe(
			'Ce temps chevauche un chantier sur lequel vous êtes encore pointé. Clôturez-le avant de déclarer ce temps.',
		)
	})

	/** "Somebody already resolved this" — the race an amend/withdraw can lose
	 * to a manager's decision landing first. */
	it('maps a resolved report refusing an amend or a withdraw', () => {
		expect(
			fieldErrorMessage('Conflict: a resolved report can no longer be amended'),
		).toBe(
			'Ce signalement a déjà été traité par un responsable, vous ne pouvez plus le modifier.',
		)
		expect(
			fieldErrorMessage(
				'Conflict: a resolved report can no longer be withdrawn',
			),
		).toBe(
			'Ce signalement a déjà été traité par un responsable, vous ne pouvez plus le retirer.',
		)
	})

	it('maps the database constraint behind a second pending report', () => {
		expect(
			fieldErrorMessage(
				'Conflict: uq_assignment_reports_pending_per_assignment',
			),
		).toBe(
			'Vous avez déjà un signalement en attente sur ce projet — modifiez-le plutôt que d’en créer un second.',
		)
	})

	/** `ApiError::Forbidden` carries no reason on the wire — only "Forbidden" —
	 * so a 403 (reporting on someone else's assignment, among others) falls
	 * through to the generic message rather than a specific one. */
	it('falls back to a generic French message for a forbidden response', () => {
		expect(fieldErrorMessage('Forbidden')).toBe(
			'Une erreur est survenue, réessayez.',
		)
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

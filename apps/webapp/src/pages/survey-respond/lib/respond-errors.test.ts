import { describe, expect, it } from 'vitest'
import { BACKEND_REASONS } from '#/pages/survey/lib/reason-messages'
import {
	classifyFailure,
	photoErrorMessage,
	RESPONDENT_REASONS,
	RespondError,
	respondentMessage,
} from '#/pages/survey-respond/lib/respond-errors'

describe('respondent reasons', () => {
	it('maps every reason a respondent can meet to a non-empty French message', () => {
		for (const reason of RESPONDENT_REASONS) {
			expect(BACKEND_REASONS).toContain(reason)
			expect(respondentMessage(reason).trim()).not.toBe('')
			expect(respondentMessage(reason)).not.toMatch(/[a-z]+_[a-z]+/)
		}
	})

	it('never echoes an unknown reason or a raw message', () => {
		expect(respondentMessage('totally_unknown')).toBe(
			'Votre réponse n’a pas pu être enregistrée.',
		)
	})
})

describe('classifyFailure', () => {
	it('tells a missing survey from an expired response', () => {
		expect(
			classifyFailure(new RespondError(404, null, null), 'survey'),
		).toEqual({ kind: 'unavailable' })
		expect(
			classifyFailure(new RespondError(404, null, null), 'response'),
		).toEqual({ kind: 'expired' })
	})

	it('turns a closing refusal into a blocking screen and a field error into an inline one', () => {
		expect(
			classifyFailure(new RespondError(409, 'survey_closed', null), 'response'),
		).toMatchObject({ kind: 'blocked', reason: 'survey_closed' })
		expect(
			classifyFailure(
				new RespondError(422, 'required_question_unanswered', 'q1'),
				'response',
			),
		).toMatchObject({ kind: 'invalid', questionId: 'q1' })
	})

	it('offers a retry for rate limits and network failures', () => {
		expect(
			classifyFailure(new RespondError(429, null, null), 'survey'),
		).toEqual({ kind: 'rate_limited' })
		expect(
			classifyFailure(new RespondError(null, null, null), 'survey'),
		).toEqual({ kind: 'network' })
		expect(classifyFailure(new TypeError('Failed to fetch'), 'survey')).toEqual(
			{
				kind: 'network',
			},
		)
	})
})

describe('photoErrorMessage', () => {
	it('explains each upload refusal in French', () => {
		expect(photoErrorMessage(new RespondError(413, null, null))).toMatch(
			/trop lourde/,
		)
		expect(photoErrorMessage(new RespondError(422, null, null))).toMatch(
			/illisible/,
		)
		expect(photoErrorMessage(new RespondError(429, null, null))).toMatch(
			/Patientez/,
		)
		expect(photoErrorMessage(new RespondError(503, null, null))).toMatch(
			/occupé/,
		)
		expect(
			photoErrorMessage(
				new RespondError(409, 'response_photo_cap_reached', null),
			),
		).toMatch(/20 photos/)
		expect(photoErrorMessage(new RespondError(null, null, null))).toMatch(
			/connexion/i,
		)
	})
})

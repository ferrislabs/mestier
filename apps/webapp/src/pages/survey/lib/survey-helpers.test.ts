import { describe, expect, it } from 'vitest'
import { answerText, answerView } from '#/pages/survey/lib/answer-format'
import {
	campaignFormErrors,
	campaignRequest,
	fromLocalInput,
} from '#/pages/survey/lib/campaign-form'
import {
	entriesOf,
	metadataErrors,
	newMetadataEntry,
	recordOf,
} from '#/pages/survey/lib/metadata'
import { publicSurveyUrl } from '#/pages/survey/lib/survey-format'

describe('metadata', () => {
	it('round-trips a record and ignores blank rows', () => {
		const entries = [...entriesOf({ session: 'S1' }), newMetadataEntry()]
		expect(recordOf(entries)).toEqual({ session: 'S1' })
		expect(metadataErrors(entries)).toEqual([])
	})

	it('refuses a missing key, a missing value and a repeated key', () => {
		expect(
			metadataErrors([
				newMetadataEntry('', 'x'),
				newMetadataEntry('a', ''),
				newMetadataEntry('b', '1'),
				newMetadataEntry('b', '2'),
			]),
		).toHaveLength(3)
	})
})

describe('campaign form', () => {
	const valid = {
		name: 'Session de mai',
		startsAt: '2026-05-01T08:00',
		endsAt: '2026-05-31T18:00',
		metadata: [],
	}

	it('accepts a window whose end follows its start, or has no end', () => {
		expect(campaignFormErrors(valid)).toEqual({})
		expect(campaignFormErrors({ ...valid, endsAt: '' })).toEqual({})
	})

	it('refuses a blank name, a missing start and an end before the start', () => {
		const errors = campaignFormErrors({
			...valid,
			name: ' ',
			endsAt: '2026-04-01T08:00',
		})
		expect(errors.name).toBeDefined()
		expect(errors.endsAt).toBe('La fin doit être après le début.')
		expect(
			campaignFormErrors({ ...valid, startsAt: '' }).startsAt,
		).toBeDefined()
	})

	it('builds the request with ISO instants', () => {
		expect(campaignRequest({ ...valid, endsAt: '' })).toEqual({
			name: 'Session de mai',
			window: { starts_at: fromLocalInput(valid.startsAt), ends_at: null },
			metadata: {},
		})
	})
})

describe('answers', () => {
	it('renders each answer kind', () => {
		expect(answerText({ Text: 'Jean' })).toBe('Jean')
		expect(answerText({ Choices: ['A', 'B'] })).toBe('A, B')
		expect(answerText({ Number: 1234.5 })).toMatch(/1\s?234,5/)
		expect(answerView({ Photos: ['k1'] })).toEqual({
			type: 'photos',
			keys: ['k1'],
		})
		expect(answerView(undefined)).toEqual({ type: 'empty' })
	})
})

describe('publicSurveyUrl', () => {
	it('builds the public link from the origin and the slug', () => {
		expect(publicSurveyUrl('https://app.mestier.fr', 'abcd1234')).toBe(
			'https://app.mestier.fr/s/abcd1234',
		)
	})
})

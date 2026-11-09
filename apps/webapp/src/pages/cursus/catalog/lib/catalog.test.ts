import { describe, expect, it } from 'vitest'
import {
	EMPTY_TRAINING,
	toTrainingRequest,
	toTrainingValues,
	validateTraining,
} from '#/pages/cursus/catalog/lib/catalog'

const TRAINING = {
	id: 'training-1',
	organization_id: 'org-1',
	name: 'BTS',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

describe('training values', () => {
	it('accepts a named training without an establishment', () => {
		expect(validateTraining({ ...EMPTY_TRAINING, name: 'CAP' })).toBeNull()
		expect(validateTraining(EMPTY_TRAINING)).toBe(
			'Le nom de la formation est obligatoire.',
		)
	})

	it('omits the establishment when the training has none', () => {
		const request = toTrainingRequest(toTrainingValues(TRAINING as never))
		expect('establishment_id' in request).toBe(false)
	})

	it('sends back the establishment the training already has', () => {
		const values = toTrainingValues({
			...TRAINING,
			establishment_id: 'est-1',
		} as never)
		expect(toTrainingRequest(values).establishment_id).toBe('est-1')
	})
})

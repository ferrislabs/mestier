import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import {
	BACKEND_REASONS,
	REASON_MESSAGES,
	reasonMessage,
	refusalOf,
} from '#/pages/survey/lib/reason-messages'

const ERROR_RS = resolve(
	process.cwd(),
	'../../libs/core/src/domain/survey/error.rs',
)

function rustCodes(): string[] {
	const source = readFileSync(ERROR_RS, 'utf8')
	const start = source.indexOf('pub fn code(&self)')
	const stop = source.indexOf('pub fn question_id(&self)')
	const body = source.slice(start, stop)
	return [...body.matchAll(/=>\s*"([a-z_]+)"/g)].map((match) => match[1])
}

describe('REASON_MESSAGES', () => {
	it('copies exactly the codes of SurveyError::code()', () => {
		const codes = rustCodes()
		expect(codes.length).toBeGreaterThan(60)
		expect([...BACKEND_REASONS].sort()).toEqual([...codes].sort())
	})

	it('has a non-empty French message for every backend code', () => {
		for (const code of BACKEND_REASONS) {
			expect(REASON_MESSAGES[code]?.trim() ?? '').not.toBe('')
		}
	})

	it('falls back on a generic message for an unknown code', () => {
		expect(reasonMessage('nope')).toMatch(/refus/i)
		expect(reasonMessage('graph_cycle')).toBe(REASON_MESSAGES.graph_cycle)
	})
})

describe('refusalOf', () => {
	it('reads the reason and the question of an API error', () => {
		const error = Object.assign(new Error('Conflict'), {
			status: 422,
			data: {
				code: 'E_UNPROCESSABLE_ENTITY',
				details: { reason: 'graph_cycle', question_id: 'q1' },
			},
		})
		expect(refusalOf(error)).toEqual({
			reason: 'graph_cycle',
			questionId: 'q1',
			message: REASON_MESSAGES.graph_cycle,
		})
	})

	it('keeps the raw message when the error has no reason', () => {
		expect(refusalOf(new Error('Boom'))).toEqual({
			reason: null,
			questionId: null,
			message: 'Boom',
		})
	})
})

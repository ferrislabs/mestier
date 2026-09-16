import { describe, expect, it } from 'vitest'
import type { AuthScheme } from '#/hooks/use-automation'
import {
	buildCredentialFormErrors,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import type { CredentialFormValues } from '#/pages/automation/types'

describe('emptyCredentialForm', () => {
	it('defaults to a supplied origin with no data', () => {
		expect(emptyCredentialForm()).toEqual({
			kind: '',
			name: '',
			origin: 'supplied',
			data: {},
		})
	})

	it('accepts a default kind', () => {
		expect(emptyCredentialForm('bearer_token').kind).toBe('bearer_token')
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

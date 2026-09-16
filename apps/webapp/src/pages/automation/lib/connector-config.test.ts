import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	isFieldVisible,
	isSelectKind,
	isSigningCredentialField,
	SIGNING_CREDENTIAL_FIELD_NAME,
	visibleFields,
	withField,
	withoutField,
} from '#/pages/automation/lib/connector-config'

function field(
	overrides: Partial<Schemas.FieldResponse> = {},
): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: overrides.name ?? 'Field',
		name: 'field',
		required: false,
		secret: false,
		visible_when: null,
		...overrides,
	}
}

describe('isFieldVisible', () => {
	it('is visible when the field carries no visible_when', () => {
		expect(isFieldVisible(field(), {})).toBe(true)
	})

	it('is visible when the controlling field holds one of the listed values', () => {
		const target = field({
			name: 'target',
			visible_when: { field: 'mode', any_of: ['advanced', 'expert'] },
		})

		expect(isFieldVisible(target, { mode: 'advanced' })).toBe(true)
		expect(isFieldVisible(target, { mode: 'expert' })).toBe(true)
	})

	it('is hidden when the controlling field holds a value outside the list', () => {
		const target = field({
			name: 'target',
			visible_when: { field: 'mode', any_of: ['advanced'] },
		})

		expect(isFieldVisible(target, { mode: 'simple' })).toBe(false)
	})

	it('is hidden when the controlling field has no value yet', () => {
		const target = field({
			name: 'target',
			visible_when: { field: 'mode', any_of: ['advanced'] },
		})

		expect(isFieldVisible(target, {})).toBe(false)
	})
})

describe('visibleFields', () => {
	it('keeps unconditional fields and fields whose condition currently holds', () => {
		const mode = field({ name: 'mode', kind: 'Select' })
		const target = field({
			name: 'target',
			visible_when: { field: 'mode', any_of: ['advanced'] },
		})
		const always = field({ name: 'always' })

		expect(visibleFields([mode, target, always], { mode: 'advanced' })).toEqual(
			[mode, target, always],
		)
		expect(visibleFields([mode, target, always], { mode: 'simple' })).toEqual([
			mode,
			always,
		])
	})
})

describe('withField / withoutField', () => {
	it('sets a field value without mutating the original config', () => {
		const config = { a: 1 }
		const next = withField(config, 'b', 2)

		expect(next).toEqual({ a: 1, b: 2 })
		expect(config).toEqual({ a: 1 })
	})

	it('removes a field value without mutating the original config', () => {
		const config = { a: 1, b: 2 }
		const next = withoutField(config, 'b')

		expect(next).toEqual({ a: 1 })
		expect(config).toEqual({ a: 1, b: 2 })
	})
})

describe('isSelectKind', () => {
	it('recognises a Select kind and exposes its options', () => {
		const kind: Schemas.FieldKindResponse = {
			Select: { options: [{ value: 'a', label: 'A' }] },
		}

		expect(isSelectKind(kind)).toBe(true)
		if (isSelectKind(kind)) {
			expect(kind.Select.options).toEqual([{ value: 'a', label: 'A' }])
		}
	})

	it('rejects every other kind', () => {
		expect(isSelectKind('Text')).toBe(false)
		expect(isSelectKind('Number')).toBe(false)
		expect(isSelectKind('Bool')).toBe(false)
		expect(isSelectKind('Json')).toBe(false)
	})
})

describe('isSigningCredentialField', () => {
	it('names exactly the signing_credential_id field, by name, not by any connector kind', () => {
		expect(
			isSigningCredentialField(field({ name: SIGNING_CREDENTIAL_FIELD_NAME })),
		).toBe(true)
		expect(isSigningCredentialField(field({ name: 'credential_id' }))).toBe(
			false,
		)
		expect(isSigningCredentialField(field({ name: 'url' }))).toBe(false)
	})
})

import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	isFieldVisible,
	isJsonMapRepresentable,
	isSelectKind,
	isSigningCredentialField,
	jsonEntriesFromValue,
	nextJsonEntryKey,
	SIGNING_CREDENTIAL_FIELD_NAME,
	valueFromJsonEntries,
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
		const mode = field({ name: 'mode' })
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

describe('isJsonMapRepresentable', () => {
	it('accepts an absent value, because an empty field starts as a map', () => {
		expect(isJsonMapRepresentable(undefined)).toBe(true)
	})

	it('accepts a plain object, empty or populated', () => {
		expect(isJsonMapRepresentable({})).toBe(true)
		expect(isJsonMapRepresentable({ a: 1 })).toBe(true)
	})

	it('rejects an array, since rows cannot represent an ordered list', () => {
		expect(isJsonMapRepresentable([1, 2, 3])).toBe(false)
	})

	it('rejects a scalar, including a string holding a whole expression', () => {
		expect(isJsonMapRepresentable('{{ trigger.items }}')).toBe(false)
		expect(isJsonMapRepresentable(42)).toBe(false)
		expect(isJsonMapRepresentable(true)).toBe(false)
		expect(isJsonMapRepresentable(null)).toBe(false)
	})
})

describe('jsonEntriesFromValue', () => {
	it('lists no entries for an absent or non-map value', () => {
		expect(jsonEntriesFromValue(undefined)).toEqual([])
		expect(jsonEntriesFromValue([1, 2])).toEqual([])
		expect(jsonEntriesFromValue('{{ trigger.items }}')).toEqual([])
	})

	it('turns each own key into an entry, string values kept verbatim', () => {
		expect(
			jsonEntriesFromValue({ 'Content-Type': 'application/json' }),
		).toEqual([{ key: 'Content-Type', value: 'application/json' }])
	})

	it('stringifies a non-string value so it stays editable as text', () => {
		expect(jsonEntriesFromValue({ retries: 3, active: true })).toEqual([
			{ key: 'retries', value: '3' },
			{ key: 'active', value: 'true' },
		])
	})
})

describe('valueFromJsonEntries', () => {
	it('is undefined for no entries, so an empty field stays absent rather than {}', () => {
		expect(valueFromJsonEntries([])).toBeUndefined()
	})

	it('drops entries whose key is blank', () => {
		expect(valueFromJsonEntries([{ key: '', value: 'orphan' }])).toBeUndefined()
		expect(
			valueFromJsonEntries([
				{ key: 'a', value: 'kept' },
				{ key: '  ', value: 'ignored' },
			]),
		).toEqual({ a: 'kept' })
	})

	it('parses a value that is valid JSON, so numbers and booleans round-trip', () => {
		expect(
			valueFromJsonEntries([
				{ key: 'retries', value: '3' },
				{ key: 'active', value: 'true' },
			]),
		).toEqual({ retries: 3, active: true })
	})

	it('keeps a value that is not valid JSON as a plain string', () => {
		expect(
			valueFromJsonEntries([
				{ key: 'Content-Type', value: 'application/json' },
			]),
		).toEqual({ 'Content-Type': 'application/json' })
	})

	it('keeps a whole expression as a string rather than trying to parse it', () => {
		expect(
			valueFromJsonEntries([
				{ key: 'Authorization', value: '{{ connectors.c1.output.token }}' },
			]),
		).toEqual({ Authorization: '{{ connectors.c1.output.token }}' })
	})
})

describe('nextJsonEntryKey', () => {
	it('proposes "key" when it is free', () => {
		expect(nextJsonEntryKey([])).toBe('key')
		expect(nextJsonEntryKey([{ key: 'other', value: 'v' }])).toBe('key')
	})

	it('numbers the proposal once "key" is already used', () => {
		expect(nextJsonEntryKey([{ key: 'key', value: 'v' }])).toBe('key_2')
		expect(
			nextJsonEntryKey([
				{ key: 'key', value: 'v' },
				{ key: 'key_2', value: 'v' },
			]),
		).toBe('key_3')
	})
})

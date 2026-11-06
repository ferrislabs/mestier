import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	isFieldVisible,
	isJsonMapRepresentable,
	isSelectKind,
	isSigningCredentialField,
	JSON_ENTRY_KINDS,
	type JsonEntryKind,
	jsonEntriesFromValue,
	jsonEntryValue,
	numberFieldError,
	numberFieldValue,
	parseRawJson,
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

describe('JSON_ENTRY_KINDS', () => {
	it('offers the four types of the prototype, in its order', () => {
		expect(JSON_ENTRY_KINDS).toEqual([
			{ value: 'string', label: 'Texte' },
			{ value: 'number', label: 'Nombre' },
			{ value: 'boolean', label: 'Booléen' },
			{ value: 'json', label: 'JSON' },
		])
	})
})

describe('jsonEntriesFromValue', () => {
	it('lists no entries for an absent or non-map value', () => {
		expect(jsonEntriesFromValue(undefined)).toEqual([])
		expect(jsonEntriesFromValue([1, 2])).toEqual([])
		expect(jsonEntriesFromValue('{{ trigger.items }}')).toEqual([])
	})

	it('turns each own key into a text entry, string values kept verbatim', () => {
		expect(
			jsonEntriesFromValue({ 'Content-Type': 'application/json' }),
		).toEqual([
			{ key: 'Content-Type', value: 'application/json', kind: 'string' },
		])
	})

	it('types a number and a boolean after the value they hold', () => {
		expect(jsonEntriesFromValue({ retries: 3, active: true })).toEqual([
			{ key: 'retries', value: '3', kind: 'number' },
			{ key: 'active', value: 'true', kind: 'boolean' },
		])
	})

	it('keeps a number written as a string typed as text', () => {
		expect(jsonEntriesFromValue({ code: '5' })).toEqual([
			{ key: 'code', value: '5', kind: 'string' },
		])
	})

	it('shows an object, a list or null as JSON text', () => {
		expect(
			jsonEntriesFromValue({ meta: { a: 1 }, tags: ['x'], none: null }),
		).toEqual([
			{ key: 'meta', value: '{"a":1}', kind: 'json' },
			{ key: 'tags', value: '["x"]', kind: 'json' },
			{ key: 'none', value: 'null', kind: 'json' },
		])
	})
})

describe('jsonEntriesFromValue with the kinds a spec declares', () => {
	const kinds: Record<string, JsonEntryKind> = {
		name: 'string',
		age: 'number',
		ok: 'boolean',
		tags: 'json',
	}

	it('types an empty key after the spec instead of showing it as null', () => {
		expect(
			jsonEntriesFromValue(
				{ name: '', age: null, ok: null, tags: null, free: null },
				kinds,
			),
		).toEqual([
			{ key: 'name', value: '', kind: 'string' },
			{ key: 'age', value: '', kind: 'number' },
			{ key: 'ok', value: '', kind: 'boolean' },
			{ key: 'tags', value: '', kind: 'json' },
			{ key: 'free', value: 'null', kind: 'json' },
		])
	})

	it('keeps an expression on a typed key typed as the spec says', () => {
		expect(jsonEntriesFromValue({ age: '{{ trigger.age }}' }, kinds)).toEqual([
			{ key: 'age', value: '{{ trigger.age }}', kind: 'number' },
		])
	})

	it('reads a value of the declared type as itself', () => {
		expect(
			jsonEntriesFromValue({ age: 3, ok: true, tags: ['a'] }, kinds),
		).toEqual([
			{ key: 'age', value: '3', kind: 'number' },
			{ key: 'ok', value: 'true', kind: 'boolean' },
			{ key: 'tags', value: '["a"]', kind: 'json' },
		])
	})

	it('writes a typed empty key back as null, as the prototype did', () => {
		expect(
			valueFromJsonEntries(
				jsonEntriesFromValue({ age: null, name: '' }, kinds),
			),
		).toEqual({ age: null, name: '' })
	})
})

describe('jsonEntryValue', () => {
	it('keeps a text entry as a string, even when it looks like a number', () => {
		expect(jsonEntryValue({ key: 'code', value: '5', kind: 'string' })).toEqual(
			{ value: '5', error: null },
		)
	})

	it('reads a number', () => {
		expect(
			jsonEntryValue({ key: 'retries', value: ' 3.5 ', kind: 'number' }),
		).toEqual({ value: 3.5, error: null })
	})

	it('flags a number that is not one', () => {
		expect(jsonEntryValue({ key: 'n', value: 'abc', kind: 'number' })).toEqual({
			value: 'abc',
			error: 'Nombre invalide.',
		})
	})

	it('reads a boolean', () => {
		expect(
			jsonEntryValue({ key: 'ok', value: 'false', kind: 'boolean' }),
		).toEqual({ value: false, error: null })
	})

	it('flags a boolean that is neither true nor false', () => {
		expect(
			jsonEntryValue({ key: 'ok', value: 'oui', kind: 'boolean' }),
		).toEqual({ value: 'oui', error: 'Saisissez true ou false.' })
	})

	it('reads a JSON value', () => {
		expect(
			jsonEntryValue({ key: 'meta', value: '{"a":1}', kind: 'json' }),
		).toEqual({ value: { a: 1 }, error: null })
	})

	it('flags JSON that does not parse', () => {
		expect(jsonEntryValue({ key: 'meta', value: '{a', kind: 'json' })).toEqual({
			value: '{a',
			error: 'JSON invalide.',
		})
	})

	it('sends null for an empty entry that is not text', () => {
		expect(jsonEntryValue({ key: 'n', value: '  ', kind: 'number' })).toEqual({
			value: null,
			error: null,
		})
	})

	it('keeps an expression as a string whatever the type, without an error', () => {
		expect(
			jsonEntryValue({
				key: 'montant',
				value: '{{ connectors.c1.output.total }}',
				kind: 'number',
			}),
		).toEqual({ value: '{{ connectors.c1.output.total }}', error: null })
	})
})

describe('valueFromJsonEntries', () => {
	it('is undefined for no entries, so an empty field stays absent rather than {}', () => {
		expect(valueFromJsonEntries([])).toBeUndefined()
	})

	it('drops entries whose key is blank', () => {
		expect(
			valueFromJsonEntries([{ key: '', value: 'orphan', kind: 'string' }]),
		).toBeUndefined()
		expect(
			valueFromJsonEntries([
				{ key: 'a', value: 'kept', kind: 'string' },
				{ key: '  ', value: 'ignored', kind: 'string' },
			]),
		).toEqual({ a: 'kept' })
	})

	it('trims the key', () => {
		expect(
			valueFromJsonEntries([{ key: ' a ', value: 'v', kind: 'string' }]),
		).toEqual({ a: 'v' })
	})

	it('types each value after its entry, so a text "5" stays a string', () => {
		expect(
			valueFromJsonEntries([
				{ key: 'retries', value: '3', kind: 'number' },
				{ key: 'active', value: 'true', kind: 'boolean' },
				{ key: 'code', value: '5', kind: 'string' },
			]),
		).toEqual({ retries: 3, active: true, code: '5' })
	})

	it('keeps a whole expression as a string rather than trying to parse it', () => {
		expect(
			valueFromJsonEntries([
				{
					key: 'Authorization',
					value: '{{ connectors.c1.output.token }}',
					kind: 'string',
				},
			]),
		).toEqual({ Authorization: '{{ connectors.c1.output.token }}' })
	})
})

describe('parseRawJson', () => {
	it('is empty for a blank text, so the field returns to absent', () => {
		expect(parseRawJson('  \n')).toEqual({ state: 'empty' })
	})

	it('parses valid JSON', () => {
		expect(parseRawJson('{"a": 1}')).toEqual({
			state: 'valid',
			value: { a: 1 },
		})
		expect(parseRawJson('[1, 2]')).toEqual({ state: 'valid', value: [1, 2] })
	})

	it('accepts a bare expression, as a drop would write it', () => {
		expect(parseRawJson(' {{ trigger.items }} ')).toEqual({
			state: 'valid',
			value: '{{ trigger.items }}',
		})
	})

	it('rejects text that is neither JSON nor an expression', () => {
		expect(parseRawJson('{"a": ')).toEqual({ state: 'invalid' })
		expect(parseRawJson('{"a": {{ x }}}')).toEqual({ state: 'invalid' })
	})
})

describe('numberFieldValue', () => {
	it('is undefined for a blank text, so the field returns to absent', () => {
		expect(numberFieldValue('')).toBeUndefined()
		expect(numberFieldValue('   ')).toBeUndefined()
	})

	it('reads a number as a number', () => {
		expect(numberFieldValue('42')).toBe(42)
		expect(numberFieldValue('-3.5')).toBe(-3.5)
	})

	it('keeps an expression as text, so a variable can feed the number', () => {
		expect(numberFieldValue('{{ connectors.c1.output.id }}')).toBe(
			'{{ connectors.c1.output.id }}',
		)
	})

	it('keeps text that is not a number as typed, for the server to refuse', () => {
		expect(numberFieldValue('abc')).toBe('abc')
	})
})

describe('numberFieldError', () => {
	it('flags text that is neither a number nor an expression', () => {
		expect(numberFieldError('abc')).toBe('Nombre invalide.')
		expect(numberFieldError('1.2.3')).toBe('Nombre invalide.')
	})

	it('accepts a blank text, a number and an expression', () => {
		expect(numberFieldError('')).toBeNull()
		expect(numberFieldError('12')).toBeNull()
		expect(numberFieldError('{{ trigger.id }}')).toBeNull()
	})
})

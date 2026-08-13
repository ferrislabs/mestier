import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { AuthField } from '#/hooks/use-automation'
import { FieldForm } from '#/pages/automation/ui/field-form'

// jsdom has no pointer-capture APIs or `scrollIntoView`; Radix's `Select`
// calls both when an item is actually selected. Scoped to this file — see
// `automation-section.test.tsx`'s identical comment for why this is not in
// `vitest.setup.ts`.
for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

function textField(overrides: Partial<AuthField> = {}): AuthField {
	return {
		name: 'url',
		label: 'URL',
		required: true,
		kind: 'Text',
		expression: false,
		secret: false,
		...overrides,
	}
}

describe('FieldForm — Text', () => {
	it('renders a text input and reports changes by field name', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<FieldForm fields={[textField()]} values={{}} onChange={onChange} />)

		await user.type(screen.getByLabelText('URL'), 'x')

		expect(onChange).toHaveBeenCalledWith('url', 'x')
	})

	it('masks a secret field as a password input', () => {
		render(
			<FieldForm
				fields={[textField({ name: 'token', label: 'Token', secret: true })]}
				values={{}}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Token').getAttribute('type')).toBe('password')
	})

	it('marks an optional field without implying the required one is', () => {
		render(
			<FieldForm
				fields={[textField({ required: false })]}
				values={{}}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByText('(optionnel)')).toBeDefined()
	})
})

describe('FieldForm — Number', () => {
	it('reports a parsed number, and undefined once cleared', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({ name: 'timeout', label: 'Timeout', kind: 'Number' }),
				]}
				values={{ timeout: 30 }}
				onChange={onChange}
			/>,
		)

		const input = screen.getByLabelText('Timeout')
		expect((input as HTMLInputElement).value).toBe('30')

		await user.clear(input)
		expect(onChange).toHaveBeenLastCalledWith('timeout', undefined)
	})
})

describe('FieldForm — Bool', () => {
	it('toggles a switch and reports a boolean', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'active',
						label: 'Actif',
						kind: 'Bool',
						required: false,
					}),
				]}
				values={{ active: false }}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByLabelText('Actif'))

		expect(onChange).toHaveBeenCalledWith('active', true)
	})
})

describe('FieldForm — Select', () => {
	it('offers exactly the catalogue-provided options, never free text', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'method',
						label: 'Method',
						kind: {
							Select: {
								options: [
									{ value: 'GET', label: 'GET' },
									{ value: 'POST', label: 'POST' },
								],
							},
						},
					}),
				]}
				values={{}}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Method' }))
		await user.click(screen.getByRole('option', { name: 'POST' }))

		expect(onChange).toHaveBeenCalledWith('method', 'POST')
	})
})

describe('FieldForm — Json', () => {
	it('parses valid JSON and reports the parsed value, not the text', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'headers',
						label: 'Headers',
						kind: 'Json',
						required: false,
					}),
				]}
				values={{}}
				onChange={onChange}
			/>,
		)

		await user.type(screen.getByLabelText('Headers'), '{{"a":1}')

		expect(onChange).toHaveBeenLastCalledWith('headers', { a: 1 })
		expect(screen.queryByText('JSON invalide')).toBeNull()
	})

	it('flags invalid JSON without calling onChange with garbage', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'headers',
						label: 'Headers',
						kind: 'Json',
						required: false,
					}),
				]}
				values={{}}
				onChange={onChange}
			/>,
		)

		await user.type(screen.getByLabelText('Headers'), '{{not json')

		expect(screen.getByText('JSON invalide')).toBeDefined()
		expect(onChange).not.toHaveBeenCalledWith('headers', expect.anything())
	})
})

describe('FieldForm — expression fields', () => {
	it('offers the insert-expression button only when the field allows it', () => {
		render(
			<FieldForm
				fields={[
					textField({
						name: 'predicate',
						label: 'Predicate',
						expression: true,
					}),
					textField({ name: 'plain', label: 'Plain' }),
				]}
				values={{}}
				onChange={vi.fn()}
				onInsertExpression={vi.fn()}
			/>,
		)

		expect(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		).toBeDefined()
		// Exactly one — the plain field gets none.
		expect(
			screen.getAllByRole('button', { name: 'Insérer une expression' }),
		).toHaveLength(1)
	})

	it('names the field it was clicked for', async () => {
		const user = userEvent.setup()
		const onInsertExpression = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'predicate',
						label: 'Predicate',
						expression: true,
					}),
				]}
				values={{}}
				onChange={vi.fn()}
				onInsertExpression={onInsertExpression}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		)

		expect(onInsertExpression).toHaveBeenCalledWith('predicate')
	})

	it('renders no insert button at all when the callback is omitted', () => {
		render(
			<FieldForm
				fields={[textField({ expression: true })]}
				values={{}}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.queryByRole('button', { name: 'Insérer une expression' }),
		).toBeNull()
	})
})

describe('FieldForm — visible_when', () => {
	const fields: AuthField[] = [
		textField({
			name: 'type',
			label: 'Type',
			kind: {
				Select: {
					options: [
						{ value: 'b2b', label: 'B2B' },
						{ value: 'b2c', label: 'B2C' },
					],
				},
			},
		}),
		textField({
			name: 'kind',
			label: 'Kind',
			visible_when: { field: 'type', any_of: ['b2b'] },
		}),
	]

	it('hides the dependent field until the condition matches', () => {
		render(
			<FieldForm fields={fields} values={{ type: 'b2c' }} onChange={vi.fn()} />,
		)

		expect(screen.queryByLabelText('Kind')).toBeNull()
	})

	it('shows the dependent field once the condition matches', () => {
		render(
			<FieldForm fields={fields} values={{ type: 'b2b' }} onChange={vi.fn()} />,
		)

		expect(screen.getByLabelText('Kind')).toBeDefined()
	})
})

describe('FieldForm — signing_credential_id', () => {
	it('renders a credential picker instead of a free-text UUID input', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<FieldForm
				fields={[
					textField({
						name: 'signing_credential_id',
						label: 'Signing credential',
						required: false,
					}),
				]}
				values={{}}
				onChange={onChange}
				signingCredentials={[{ id: 'cred-1', name: 'Signature sortante' }]}
			/>,
		)

		expect(
			screen.queryByRole('textbox', { name: 'Signing credential' }),
		).toBeNull()

		await user.click(
			screen.getByRole('combobox', { name: 'Signing credential' }),
		)
		await user.click(screen.getByRole('option', { name: 'Signature sortante' }))

		expect(onChange).toHaveBeenCalledWith('signing_credential_id', 'cred-1')
	})
})

describe('FieldForm — validation errors', () => {
	it('shows the backend error under the field it names', () => {
		render(
			<FieldForm
				fields={[textField()]}
				values={{}}
				onChange={vi.fn()}
				errors={{ url: 'URL invalide' }}
			/>,
		)

		expect(screen.getByText('URL invalide')).toBeDefined()
	})
})

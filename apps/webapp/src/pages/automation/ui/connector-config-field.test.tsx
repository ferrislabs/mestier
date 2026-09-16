import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { ConnectorConfigField } from '#/pages/automation/ui/connector-config-field'

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

function field(
	overrides: Partial<Schemas.FieldResponse> = {},
): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: 'Champ',
		name: 'field',
		required: false,
		secret: false,
		visible_when: null,
		...overrides,
	}
}

describe('ConnectorConfigField — Text', () => {
	it('renders the current value and reports typed changes', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL' })}
				value="https://a"
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		const input = screen.getByLabelText('URL') as HTMLInputElement
		expect(input.value).toBe('https://a')

		await user.type(input, 'x')

		expect(onChange).toHaveBeenLastCalledWith('https://ax')
	})

	it('masks a secret field behind a password input', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'token', label: 'Token', secret: true })}
				value="s3cr3t"
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Token').getAttribute('type')).toBe('password')
	})
})

describe('ConnectorConfigField — Number', () => {
	it('reports a typed number as a JS number, not a string', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'timeout', label: 'Timeout', kind: 'Number' })}
				value={undefined}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Timeout'), {
			target: { value: '42' },
		})

		expect(onChange).toHaveBeenLastCalledWith(42)
	})

	it('reports undefined once the field is cleared', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'timeout', label: 'Timeout', kind: 'Number' })}
				value={42}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.clear(screen.getByLabelText('Timeout'))

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})
})

describe('ConnectorConfigField — Bool', () => {
	it('toggles between true and false', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'active', label: 'Actif', kind: 'Bool' })}
				value={false}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.click(screen.getByLabelText('Actif'))

		expect(onChange).toHaveBeenLastCalledWith(true)
	})
})

describe('ConnectorConfigField — Select', () => {
	it('offers the descriptor-declared options and reports the chosen value', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({
					name: 'method',
					label: 'Méthode',
					kind: {
						Select: {
							options: [
								{ value: 'GET', label: 'GET' },
								{ value: 'POST', label: 'POST' },
							],
						},
					},
				})}
				value="GET"
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		await user.click(await screen.findByRole('option', { name: 'POST' }))

		expect(onChange).toHaveBeenCalledWith('POST')
	})
})

describe('ConnectorConfigField — Json', () => {
	it('reports the parsed value once the text is valid JSON', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={undefined}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Headers'), {
			target: { value: '{"a":1}' },
		})

		expect(onChange).toHaveBeenLastCalledWith({ a: 1 })
	})

	it('flags invalid JSON locally instead of propagating garbage', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={undefined}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Headers'), {
			target: { value: '{not json' },
		})

		expect(onChange).not.toHaveBeenCalled()
		expect(screen.getByText(/JSON invalide/i)).toBeDefined()
	})
})

describe('ConnectorConfigField — validation error', () => {
	it('shows the message next to the field it names', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL' })}
				value=""
				error="URL manquante"
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(screen.getByText('URL manquante')).toBeDefined()
		expect(screen.getByLabelText('URL').getAttribute('aria-invalid')).toBe(
			'true',
		)
	})

	it('shows no error when there is none', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL' })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('alert')).toBeNull()
	})
})

describe('ConnectorConfigField — expression affordance', () => {
	it('offers it when the field accepts an expression, and calls back on click', async () => {
		const user = userEvent.setup()
		const onOpenExpression = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={onOpenExpression}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /URL/ }))

		expect(onOpenExpression).toHaveBeenCalledTimes(1)
	})

	it('is absent when the field does not accept an expression', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'method', label: 'Méthode', expression: false })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('button')).toBeNull()
	})
})

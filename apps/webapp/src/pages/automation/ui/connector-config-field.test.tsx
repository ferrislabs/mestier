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

describe('ConnectorConfigField — Json — the empty state', () => {
	it('shows an empty zone and a way to add an entry, no bare textarea', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={undefined}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('textbox')).toBeNull()
		expect(
			screen.getByRole('button', { name: /ajouter une entrée/i }),
		).toBeDefined()
	})

	it('adding an entry writes a first key/value pair into the field', async () => {
		const user = userEvent.setup()
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

		await user.click(
			screen.getByRole('button', { name: /ajouter une entrée/i }),
		)

		expect(onChange).toHaveBeenLastCalledWith({ key: 'value' })
	})
})

describe('ConnectorConfigField — Json — rows', () => {
	it('renders one row per entry, with a key input and a value input', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={{ 'Content-Type': 'application/json' }}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(
			screen.getByDisplayValue('Content-Type') as HTMLInputElement,
		).toBeDefined()
		expect(
			screen.getByDisplayValue('application/json') as HTMLInputElement,
		).toBeDefined()
	})

	it('removing the last entry returns the field to its empty state, not {}', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={{ 'Content-Type': 'application/json' }}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /supprimer l'entrée/i }),
		)

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})

	it('accepts a dropped expression on a value cell, scoped to that row', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({
					name: 'headers',
					label: 'Headers',
					kind: 'Json',
					expression: true,
				})}
				value={{ Authorization: '' }}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		const dataTransfer = { getData: () => 'connectors.c1.output.token' }
		fireEvent.drop(screen.getByDisplayValue(''), { dataTransfer })

		expect(onChange).toHaveBeenLastCalledWith({
			Authorization: '{{ connectors.c1.output.token }}',
		})
	})

	it('ignores a dropped expression when the field does not accept one', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({
					name: 'headers',
					label: 'Headers',
					kind: 'Json',
					expression: false,
				})}
				value={{ Authorization: '' }}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		const dataTransfer = { getData: () => 'connectors.c1.output.token' }
		fireEvent.drop(screen.getByDisplayValue(''), { dataTransfer })

		expect(onChange).not.toHaveBeenCalled()
	})
})

describe('ConnectorConfigField — Json — array and scalar values stay raw', () => {
	it('renders an array value as raw JSON, since rows cannot represent it', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'items', label: 'Items', kind: 'Json' })}
				value={[1, 2, 3]}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect((screen.getByLabelText('Items') as HTMLTextAreaElement).value).toBe(
			'[\n  1,\n  2,\n  3\n]',
		)
		expect(
			screen.queryByRole('button', { name: /ajouter une entrée/i }),
		).toBeNull()
	})

	it('renders a whole expression as raw JSON, not as a single-entry row', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'items', label: 'Items', kind: 'Json' })}
				value="{{ trigger.items }}"
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect((screen.getByLabelText('Items') as HTMLTextAreaElement).value).toBe(
			'{{ trigger.items }}',
		)
		expect(
			screen.queryByRole('button', { name: /ajouter une entrée/i }),
		).toBeNull()
	})

	it('reports the parsed value once raw text is valid JSON', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'items', label: 'Items', kind: 'Json' })}
				value={[1]}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Items'), {
			target: { value: '[1,2]' },
		})

		expect(onChange).toHaveBeenLastCalledWith([1, 2])
	})

	it('flags invalid JSON locally instead of propagating garbage', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'items', label: 'Items', kind: 'Json' })}
				value={[1]}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Items'), {
			target: { value: '[not json' },
		})

		expect(onChange).not.toHaveBeenCalled()
		expect(screen.getByText(/JSON invalide/i)).toBeDefined()
	})
})

describe('ConnectorConfigField — Json — switching between rows and raw', () => {
	it('keeps the content when switching to raw and back to rows', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={{ a: '1', b: '2' }}
				error={null}
				onChange={onChange}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /passer en json/i }))
		expect(
			(screen.getByLabelText('Headers') as HTMLTextAreaElement).value,
		).toBe(JSON.stringify({ a: '1', b: '2' }, null, 2))

		await user.click(
			screen.getByRole('button', { name: /revenir aux champs/i }),
		)

		expect(screen.getByDisplayValue('a')).toBeDefined()
		expect(screen.getByDisplayValue('b')).toBeDefined()
		expect(onChange).not.toHaveBeenCalled()
	})

	it('offers no raw-to-rows switch once the raw text is an array', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorConfigField
				field={field({ name: 'headers', label: 'Headers', kind: 'Json' })}
				value={{}}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /passer en json/i }))
		fireEvent.change(screen.getByLabelText('Headers'), {
			target: { value: '[1,2]' },
		})

		expect(
			screen.queryByRole('button', { name: /revenir aux champs/i }),
		).toBeNull()
	})
})

describe('ConnectorConfigField — Json — external updates', () => {
	it('resyncs the visible text when the value changes from outside the field, as an inserted expression would', () => {
		const jsonField = field({ name: 'headers', label: 'Headers', kind: 'Json' })
		const { rerender } = render(
			<ConnectorConfigField
				field={jsonField}
				value={undefined}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		rerender(
			<ConnectorConfigField
				field={jsonField}
				value="{{ trigger.headers }}"
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		expect(
			(screen.getByLabelText('Headers') as HTMLTextAreaElement).value,
		).toBe('{{ trigger.headers }}')
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

	it('shows a visible label naming what it does, not a bare glyph', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
			/>,
		)

		const button = screen.getByRole('button', { name: /URL/ })
		expect(button.textContent?.trim().length).toBeGreaterThan(0)
		expect(button.getAttribute('title')?.length).toBeGreaterThan(0)
	})
})

describe('ConnectorConfigField — the control ref', () => {
	it('hands back the live text input so a caller can read its cursor', () => {
		let el: HTMLInputElement | HTMLTextAreaElement | null = null
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', kind: 'Text' })}
				value="https://a"
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
				controlRef={(node) => {
					el = node
				}}
			/>,
		)

		expect(el).toBe(screen.getByLabelText('URL'))
	})

	it('hands back the live textarea for a Json field rendered raw', () => {
		let el: HTMLInputElement | HTMLTextAreaElement | null = null
		render(
			<ConnectorConfigField
				field={field({ name: 'items', label: 'Items', kind: 'Json' })}
				value={[1, 2]}
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
				controlRef={(node) => {
					el = node
				}}
			/>,
		)

		expect(el).toBe(screen.getByLabelText('Items'))
	})
})

describe('ConnectorConfigField — dropping a datum', () => {
	it('reports the dropped path on an expression-enabled field', () => {
		const onDropExpression = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
				onDropExpression={onDropExpression}
			/>,
		)

		const dataTransfer = { getData: () => 'trigger.customer.id' }
		fireEvent.drop(screen.getByLabelText('URL'), { dataTransfer })

		expect(onDropExpression).toHaveBeenCalledWith('trigger.customer.id')
	})

	it('does nothing on a field that does not accept an expression', () => {
		const onDropExpression = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'method', label: 'Méthode', expression: false })}
				value=""
				error={null}
				onChange={vi.fn()}
				onOpenExpression={vi.fn()}
				onDropExpression={onDropExpression}
			/>,
		)

		const dataTransfer = { getData: () => 'trigger.customer.id' }
		fireEvent.drop(screen.getByLabelText('Méthode'), { dataTransfer })

		expect(onDropExpression).not.toHaveBeenCalled()
	})
})

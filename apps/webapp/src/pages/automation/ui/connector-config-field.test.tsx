import { fireEvent, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { DisplayField } from '#/pages/automation/lib/connector-field-copy'
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

function field(overrides: Partial<DisplayField> = {}): DisplayField {
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
				issues={[]}
				onChange={onChange}
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
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Token').getAttribute('type')).toBe('password')
	})
})

describe('ConnectorConfigField — Text — copy', () => {
	it('shows the placeholder of an empty field', () => {
		render(
			<ConnectorConfigField
				field={field({
					name: 'ticket_id',
					label: 'Ticket',
					placeholder: 'Glissez une variable depuis la colonne Entrée',
				})}
				value=""
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByPlaceholderText(
				'Glissez une variable depuis la colonne Entrée',
			),
		).toBe(screen.getByLabelText('Ticket'))
	})

	it('shows the hint under the field, after its issues', () => {
		render(
			<ConnectorConfigField
				field={field({
					name: 'predicate',
					label: 'Condition',
					hint: 'Le flux continue par « vrai » si la condition est remplie.',
				})}
				value=""
				issues={['Condition manquante']}
				onChange={vi.fn()}
			/>,
		)

		const issue = screen.getByText('Condition manquante')
		const hint = screen.getByText(
			'Le flux continue par « vrai » si la condition est remplie.',
		)
		expect(
			issue.compareDocumentPosition(hint) & Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})

	it('renders a multiline field as a textarea of the declared height', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'message', label: 'Message', rows: 4 })}
				value="Bonjour"
				issues={[]}
				onChange={onChange}
			/>,
		)

		const area = screen.getByLabelText('Message') as HTMLTextAreaElement
		expect(area.tagName).toBe('TEXTAREA')
		expect(area.rows).toBe(4)

		await user.type(area, '!')

		expect(onChange).toHaveBeenLastCalledWith('Bonjour!')
	})

	it('lets a variable be dropped on a multiline field', () => {
		render(
			<ConnectorConfigField
				field={field({
					name: 'message',
					label: 'Message',
					rows: 4,
					expression: true,
				})}
				value=""
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Message').className).toContain(
			'expression-drop',
		)
	})
})

describe('ConnectorConfigField — Number', () => {
	it('reports a typed number as a JS number, not a string', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={field({ name: 'timeout', label: 'Timeout', kind: 'Number' })}
				value={undefined}
				issues={[]}
				onChange={onChange}
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
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.clear(screen.getByLabelText('Timeout'))

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})
})

describe('ConnectorConfigField — Number — accepting an expression', () => {
	const amount = () =>
		field({
			name: 'amount',
			label: 'Montant',
			kind: 'Number',
			expression: true,
		})

	it('reports a typed number as a JS number', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Montant'), {
			target: { value: '42.5' },
		})

		expect(onChange).toHaveBeenLastCalledWith(42.5)
	})

	it('reports a typed expression as text', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Montant'), {
			target: { value: '{{ trigger.total }}' },
		})

		expect(onChange).toHaveBeenLastCalledWith('{{ trigger.total }}')
	})

	it('is a text input that lets a variable be dropped, unlike a native number input', () => {
		render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		const input = screen.getByLabelText('Montant')
		expect(input.getAttribute('type')).toBe('text')
		expect(input.className).toContain('expression-drop')
	})

	it('shows the expression a drop wrote, and tags it', () => {
		const { rerender } = render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		rerender(
			<ConnectorConfigField
				field={amount()}
				value="{{ connectors.c1.output.total }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect((screen.getByLabelText('Montant') as HTMLInputElement).value).toBe(
			'{{ connectors.c1.output.total }}',
		)
		expect(screen.getByText('Expression')).toBeDefined()
	})

	it('keeps a decimal being typed instead of rounding it away', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={onChange}
			/>,
		)

		const input = screen.getByLabelText('Montant') as HTMLInputElement
		await user.type(input, '1.')

		expect(input.value).toBe('1.')
		expect(onChange).toHaveBeenLastCalledWith(1)
	})

	it('reports undefined once cleared', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={amount()}
				value={42}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.clear(screen.getByLabelText('Montant'))

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})

	it('flags text that is neither a number nor an expression', () => {
		render(
			<ConnectorConfigField
				field={amount()}
				value={undefined}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Montant'), {
			target: { value: 'abc' },
		})

		expect(screen.getByText('Nombre invalide.')).toBeDefined()
	})

	it('stays a native number input when the field takes no expression', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'timeout', label: 'Délai', kind: 'Number' })}
				value={3}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Délai').getAttribute('type')).toBe('number')
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
				issues={[]}
				onChange={onChange}
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
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		await user.click(await screen.findByRole('option', { name: 'POST' }))

		expect(onChange).toHaveBeenCalledWith('POST')
	})
})

function jsonField(overrides: Partial<DisplayField> = {}): DisplayField {
	return field({
		name: 'headers',
		label: 'En-têtes',
		kind: 'Json',
		...overrides,
	})
}

describe('ConnectorConfigField — Json — the empty state', () => {
	it('shows only the way to add a field, no bare textarea', () => {
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={undefined}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('textbox')).toBeNull()
		expect(
			screen.getByRole('button', { name: 'Ajouter un champ' }),
		).toBeDefined()
		expect(screen.queryByText('Clé')).toBeNull()
	})

	it('adding a field shows an empty row without writing anything yet', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={undefined}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Ajouter un champ' }))

		expect(
			(screen.getByLabelText('Clé du champ 1') as HTMLInputElement).value,
		).toBe('')
		expect(
			(screen.getByLabelText('Valeur de champ 1') as HTMLInputElement)
				.placeholder,
		).toBe('valeur')
		expect(onChange).not.toHaveBeenCalled()
	})

	it('writes the first pair once the row has a key', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={undefined}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Ajouter un champ' }))
		fireEvent.change(screen.getByLabelText('Clé du champ 1'), {
			target: { value: 'Accept' },
		})
		fireEvent.change(screen.getByLabelText('Valeur de Accept'), {
			target: { value: 'application/json' },
		})

		expect(onChange).toHaveBeenLastCalledWith({ Accept: 'application/json' })
	})
})

describe('ConnectorConfigField — Json — rows', () => {
	it('lists the columns, then one row per entry with key, type and value', () => {
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ 'Content-Type': 'application/json' }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		const group = screen.getByRole('group', { name: 'En-têtes' })
		expect(within(group).getByText('Clé')).toBeDefined()
		expect(within(group).getByText('Type')).toBeDefined()
		expect(within(group).getByText('Valeur')).toBeDefined()
		expect(
			(within(group).getByLabelText('Clé du Content-Type') as HTMLInputElement)
				.value,
		).toBe('Content-Type')
		expect(
			(
				within(group).getByLabelText(
					'Valeur de Content-Type',
				) as HTMLInputElement
			).value,
		).toBe('application/json')
		expect(
			within(group).getByLabelText('Type de Content-Type').textContent,
		).toBe('Texte')
	})

	it('types each row after the value it holds', () => {
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ retries: 3, active: true, meta: { a: 1 } }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('Type de retries').textContent).toBe('Nombre')
		expect(screen.getByLabelText('Type de active').textContent).toBe('Booléen')
		expect(screen.getByLabelText('Type de meta').textContent).toBe('JSON')
		expect(
			(screen.getByLabelText('Valeur de active') as HTMLInputElement)
				.placeholder,
		).toBe('true ou false')
	})

	it('keeps a text "5" a string instead of turning it into a number', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ code: 'a' }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Valeur de code'), {
			target: { value: '5' },
		})

		expect(onChange).toHaveBeenLastCalledWith({ code: '5' })
	})

	it('changing the type of a row retypes its value', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ retries: '3' }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByLabelText('Type de retries'))
		await user.click(await screen.findByRole('option', { name: 'Nombre' }))

		expect(onChange).toHaveBeenLastCalledWith({ retries: 3 })
	})

	it('shows the error of a value that does not fit its type', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ retries: 3 }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Valeur de retries'), {
			target: { value: 'beaucoup' },
		})

		expect(screen.getByText('Nombre invalide.')).toBeDefined()
		expect(
			screen.getByLabelText('Valeur de retries').getAttribute('aria-invalid'),
		).toBe('true')
	})

	it('removing the last row returns the field to its empty state, not {}', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ 'Content-Type': 'application/json' }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Supprimer Content-Type' }),
		)

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})

	it('lets a variable be dropped on a value cell when the field takes one', () => {
		render(
			<ConnectorConfigField
				field={jsonField({ expression: true })}
				value={{ Authorization: '' }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByLabelText('Valeur de Authorization').className,
		).toContain('expression-drop')
	})

	it('does not offer the drop when the field takes no expression', () => {
		render(
			<ConnectorConfigField
				field={jsonField({ expression: false })}
				value={{ Authorization: '' }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByLabelText('Valeur de Authorization').className,
		).not.toContain('expression-drop')
	})

	it('takes in what the browser drops in the cell, like a typed value', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField({ expression: true })}
				value={{ Authorization: '' }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(screen.getByLabelText('Valeur de Authorization'), {
			target: { value: '{{ connectors.c1.output.token }}' },
		})

		expect(onChange).toHaveBeenLastCalledWith({
			Authorization: '{{ connectors.c1.output.token }}',
		})
	})
})

describe('ConnectorConfigField — Json — array and scalar values stay raw', () => {
	const items = () => field({ name: 'items', label: 'Éléments', kind: 'Json' })

	it('renders an array value as raw JSON, since rows cannot represent it', () => {
		render(
			<ConnectorConfigField
				field={items()}
				value={[1, 2, 3]}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			(
				screen.getByRole('textbox', {
					name: 'Éléments en JSON',
				}) as HTMLTextAreaElement
			).value,
		).toBe('[\n  1,\n  2,\n  3\n]')
		expect(
			screen.queryByRole('button', { name: 'Ajouter un champ' }),
		).toBeNull()
	})

	it('explains why the Champs mode is not available', () => {
		render(
			<ConnectorConfigField
				field={items()}
				value={[1, 2, 3]}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByText('Le mode Champs demande un objet JSON valide.'),
		).toBeDefined()
		expect(
			screen.getByRole('radio', { name: 'Champs' }).hasAttribute('disabled'),
		).toBe(true)
	})

	it('renders a whole expression as raw JSON, not as a single-entry row', () => {
		render(
			<ConnectorConfigField
				field={items()}
				value="{{ trigger.items }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			(
				screen.getByRole('textbox', {
					name: 'Éléments en JSON',
				}) as HTMLTextAreaElement
			).value,
		).toBe('{{ trigger.items }}')
		expect(
			screen.queryByRole('button', { name: 'Ajouter un champ' }),
		).toBeNull()
	})

	it('reports the parsed value once raw text is valid JSON', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={items()}
				value={[1]}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(
			screen.getByRole('textbox', { name: 'Éléments en JSON' }),
			{
				target: { value: '[1,2]' },
			},
		)

		expect(onChange).toHaveBeenLastCalledWith([1, 2])
	})

	it('accepts a bare expression typed by hand, as a drop would write it', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={items()}
				value={[1]}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(
			screen.getByRole('textbox', { name: 'Éléments en JSON' }),
			{
				target: { value: '{{ trigger.items }}' },
			},
		)

		expect(onChange).toHaveBeenLastCalledWith('{{ trigger.items }}')
		expect(screen.queryByText(/JSON invalide/)).toBeNull()
	})

	it('flags invalid JSON locally instead of propagating garbage', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={items()}
				value={[1]}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(
			screen.getByRole('textbox', { name: 'Éléments en JSON' }),
			{
				target: { value: '[not json' },
			},
		)

		expect(onChange).not.toHaveBeenCalled()
		expect(
			screen.getByText(
				'JSON invalide : vérifiez les guillemets, les virgules et les accolades. Une expression doit être entre guillemets.',
			),
		).toBeDefined()
	})

	it('reports undefined once the raw text is emptied', () => {
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={items()}
				value={[1]}
				issues={[]}
				onChange={onChange}
			/>,
		)

		fireEvent.change(
			screen.getByRole('textbox', { name: 'Éléments en JSON' }),
			{
				target: { value: '  ' },
			},
		)

		expect(onChange).toHaveBeenLastCalledWith(undefined)
	})
})

describe('ConnectorConfigField — Json — switching between fields and JSON', () => {
	it('names the toggle after the body for the body field', () => {
		render(
			<ConnectorConfigField
				field={jsonField({ name: 'body', label: 'Corps de la requête' })}
				value={{ a: '1' }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByRole('group', { name: 'Mode de saisie du corps' }),
		).toBeDefined()
	})

	it('keeps the content when switching to JSON and back to fields', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ a: '1', b: '2' }}
				issues={[]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('radio', { name: 'JSON' }))
		expect(
			(
				screen.getByRole('textbox', {
					name: 'En-têtes en JSON',
				}) as HTMLTextAreaElement
			).value,
		).toBe(JSON.stringify({ a: '1', b: '2' }, null, 2))

		await user.click(screen.getByRole('radio', { name: 'Champs' }))

		expect(screen.getByDisplayValue('a')).toBeDefined()
		expect(screen.getByDisplayValue('b')).toBeDefined()
		expect(onChange).not.toHaveBeenCalled()
	})

	it('disables the way back to fields once the JSON text is an array', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{}}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('radio', { name: 'JSON' }))
		fireEvent.change(
			screen.getByRole('textbox', { name: 'En-têtes en JSON' }),
			{
				target: { value: '[1,2]' },
			},
		)

		expect(
			screen.getByRole('radio', { name: 'Champs' }).hasAttribute('disabled'),
		).toBe(true)
	})

	it('disables the way back to fields while the JSON text is invalid', async () => {
		const user = userEvent.setup()
		render(
			<ConnectorConfigField
				field={jsonField()}
				value={{ a: '1' }}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('radio', { name: 'JSON' }))
		fireEvent.change(
			screen.getByRole('textbox', { name: 'En-têtes en JSON' }),
			{
				target: { value: '{"a": ' },
			},
		)

		expect(
			screen.getByRole('radio', { name: 'Champs' }).hasAttribute('disabled'),
		).toBe(true)
	})
})

describe('ConnectorConfigField — Json — external updates', () => {
	it('resyncs the visible text when the value changes from outside the field, as an inserted expression would', () => {
		const items = field({ name: 'items', label: 'Éléments', kind: 'Json' })
		const { rerender } = render(
			<ConnectorConfigField
				field={items}
				value={undefined}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		rerender(
			<ConnectorConfigField
				field={items}
				value="{{ trigger.items }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			(
				screen.getByRole('textbox', {
					name: 'Éléments en JSON',
				}) as HTMLTextAreaElement
			).value,
		).toBe('{{ trigger.items }}')
	})
})

describe('ConnectorConfigField — validation error', () => {
	it('shows the message next to the field it names', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL' })}
				value=""
				issues={['URL manquante']}
				onChange={vi.fn()}
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
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('alert')).toBeNull()
	})
})

describe('ConnectorConfigField — expressions', () => {
	it('tags an expression-capable field whose value holds an expression', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value="{{ trigger.id }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByText('Expression')).toBeDefined()
	})

	it('shows no tag for a plain value or a field that takes no expression', () => {
		const { rerender } = render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value="https://a"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.queryByText('Expression')).toBeNull()

		rerender(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: false })}
				value="{{ trigger.id }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.queryByText('Expression')).toBeNull()
	})

	it('shows the preview under an expression value only', () => {
		const { rerender } = render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value="{{ trigger.id }}"
				issues={[]}
				preview={<p>Aperçu : 7</p>}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.getByText('Aperçu : 7')).toBeDefined()

		rerender(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value="https://a"
				issues={[]}
				preview={<p>Aperçu : 7</p>}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.queryByText('Aperçu : 7')).toBeNull()
	})

	it('has no button of its own', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value=""
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('button')).toBeNull()
	})
})

describe('ConnectorConfigField — the drop target', () => {
	it('marks a field that takes an expression as a place to drop a variable', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value=""
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('URL').className).toContain('expression-drop')
	})

	it('does not mark a field that does not accept an expression', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: false })}
				value=""
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('URL').className).not.toContain(
			'expression-drop',
		)
	})

	it('shows an expression in monospace', () => {
		render(
			<ConnectorConfigField
				field={field({ name: 'url', label: 'URL', expression: true })}
				value="{{ trigger.id }}"
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByLabelText('URL').className).toContain('font-mono')
	})

	it('marks the raw JSON zone of a field that takes an expression', () => {
		render(
			<ConnectorConfigField
				field={field({
					name: 'items',
					label: 'Éléments',
					kind: 'Json',
					expression: true,
				})}
				value={[1]}
				issues={[]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByRole('textbox', { name: 'Éléments en JSON' }).className,
		).toContain('expression-drop')
	})
})

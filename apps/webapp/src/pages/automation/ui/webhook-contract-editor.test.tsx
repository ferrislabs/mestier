import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { ContractKey } from '#/pages/automation/lib/webhook-contract'
import { WebhookContractEditor } from '#/pages/automation/ui/webhook-contract-editor'

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

function Harness({
	initial = [],
	keyErrors = {},
	onChange = vi.fn(),
}: {
	initial?: ContractKey[]
	keyErrors?: Record<string, string>
	onChange?: (contract: ContractKey[]) => void
}) {
	const [contract, setContract] = useState(initial)
	return (
		<WebhookContractEditor
			controlId="contract"
			contract={contract}
			keyErrors={keyErrors}
			onChange={(next) => {
				setContract(next)
				onChange(next)
			}}
		/>
	)
}

const FACTURE: ContractKey = {
	name: 'facture',
	kind: 'Text',
	required: true,
	example: 'FA-1',
}
const MONTANT: ContractKey = {
	name: 'montant',
	kind: 'Number',
	required: false,
	example: '2340',
}

describe('WebhookContractEditor', () => {
	it('says no field is described yet, with no header', () => {
		render(<Harness />)

		expect(
			screen.getByText(
				'Aucun champ décrit : les nœuds suivants ne connaîtront pas la forme du corps reçu.',
			),
		).toBeDefined()
		expect(screen.queryByText('Clé')).toBeNull()
	})

	it('shows the columns Clé, Type, Requis and Exemple over the keys', () => {
		render(<Harness initial={[FACTURE, MONTANT]} />)

		for (const header of ['Clé', 'Type', 'Requis', 'Exemple'])
			expect(screen.getByText(header)).toBeDefined()
		expect(
			(screen.getByLabelText('Clé facture') as HTMLInputElement).value,
		).toBe('facture')
		expect(
			(screen.getByLabelText('Exemple pour montant') as HTMLInputElement).value,
		).toBe('2340')
		expect(
			screen
				.getByRole('switch', { name: 'facture requis' })
				.getAttribute('aria-checked'),
		).toBe('true')
		expect(
			screen
				.getByRole('switch', { name: 'montant requis' })
				.getAttribute('aria-checked'),
		).toBe('false')
	})

	it('adds a required text key', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness initial={[FACTURE]} onChange={onChange} />)

		await user.click(screen.getByRole('button', { name: 'Ajouter un champ' }))

		expect(onChange).toHaveBeenLastCalledWith([
			FACTURE,
			{ name: '', kind: 'Text', required: true, example: '' },
		])
	})

	it('renames a key, changes its example and its requirement', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness initial={[FACTURE]} onChange={onChange} />)

		await user.type(screen.getByLabelText('Clé facture'), '_numero')
		await user.clear(screen.getByLabelText('Exemple pour facture_numero'))
		await user.type(screen.getByLabelText('Exemple pour facture_numero'), 'X')
		await user.click(
			screen.getByRole('switch', { name: 'facture_numero requis' }),
		)

		expect(onChange).toHaveBeenLastCalledWith([
			{ name: 'facture_numero', kind: 'Text', required: false, example: 'X' },
		])
	})

	it('offers exactly the five types of the API', async () => {
		const user = userEvent.setup()
		render(<Harness initial={[FACTURE]} />)

		await user.click(screen.getByRole('combobox', { name: 'Type de facture' }))

		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Texte', 'Nombre', 'Booléen', 'Objet', 'Liste'])
	})

	it('changes the type of a key', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness initial={[FACTURE]} onChange={onChange} />)

		await user.click(screen.getByRole('combobox', { name: 'Type de facture' }))
		await user.click(await screen.findByRole('option', { name: 'Liste' }))

		expect(onChange).toHaveBeenLastCalledWith([{ ...FACTURE, kind: 'List' }])
	})

	it('removes the key of the row, not another one with the same name', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<Harness
				initial={[FACTURE, MONTANT, { ...FACTURE, example: 'FA-2' }]}
				onChange={onChange}
			/>,
		)

		await user.click(
			screen.getAllByRole('button', {
				name: 'Retirer facture',
			})[1] as HTMLElement,
		)

		expect(onChange).toHaveBeenLastCalledWith([FACTURE, MONTANT])
	})

	it('keeps the typing of a row when another row is removed', async () => {
		const user = userEvent.setup()
		render(<Harness initial={[FACTURE, MONTANT]} />)

		await user.click(screen.getByRole('button', { name: 'Retirer facture' }))

		expect(screen.queryByLabelText('Clé facture')).toBeNull()
		expect(
			(screen.getByLabelText('Clé montant') as HTMLInputElement).value,
		).toBe('montant')
	})

	it('explains a key without a name, a bad name, a duplicate and a bad example', () => {
		render(
			<Harness
				initial={[
					{ name: '', kind: 'Text', required: true, example: '' },
					{ name: 'mon montant', kind: 'Text', required: true, example: '' },
					FACTURE,
					FACTURE,
					{ name: 'n', kind: 'Number', required: true, example: 'beaucoup' },
				]}
			/>,
		)

		const alerts = screen
			.getAllByRole('alert')
			.map((alert) => alert.textContent)
		expect(alerts).toEqual([
			'Donnez un nom à cette clé.',
			'Lettres, chiffres, « _ » et « - » uniquement.',
			'Clé en double à ce niveau.',
			'Exemple invalide pour le type « Nombre ».',
		])
	})

	it('shows the error the server gave for a key', () => {
		render(
			<Harness
				initial={[FACTURE]}
				keyErrors={{ facture: 'Clé en double à ce niveau.' }}
			/>,
		)

		expect(screen.getByRole('alert').textContent).toBe(
			'Clé en double à ce niveau.',
		)
	})

	it('infers the keys from a JSON example and replaces the current ones', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness initial={[FACTURE]} onChange={onChange} />)

		await user.click(
			screen.getByRole('button', { name: 'Déduire d’un exemple JSON' }),
		)
		await user.click(screen.getByLabelText('Exemple de corps reçu'))
		await user.paste('{"total": 12, "client": {"nom": "A"}}')
		await user.click(screen.getByRole('button', { name: 'Déduire les champs' }))

		expect(onChange).toHaveBeenLastCalledWith([
			{ name: 'total', kind: 'Number', required: true, example: '12' },
			{
				name: 'client',
				kind: 'Object',
				required: true,
				example: '{"nom":"A"}',
			},
		])
		expect(screen.queryByLabelText('Clé facture')).toBeNull()
		expect(screen.getByLabelText('Clé total')).toBeDefined()
	})

	it('says why a JSON example cannot be used', async () => {
		const user = userEvent.setup()
		render(<Harness />)

		await user.click(
			screen.getByRole('button', { name: 'Déduire d’un exemple JSON' }),
		)
		await user.click(screen.getByLabelText('Exemple de corps reçu'))
		await user.paste('{"a": ')
		await user.click(screen.getByRole('button', { name: 'Déduire les champs' }))
		expect(
			screen.getByText(
				'JSON invalide : vérifiez les guillemets et les virgules.',
			),
		).toBeDefined()

		await user.clear(screen.getByLabelText('Exemple de corps reçu'))
		await user.paste('[1, 2]')
		await user.click(screen.getByRole('button', { name: 'Déduire les champs' }))
		expect(
			screen.getByText('Collez un objet JSON, par exemple { "montant": 120 }.'),
		).toBeDefined()
	})

	it('drops the import without changing anything', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness onChange={onChange} />)

		await user.click(
			screen.getByRole('button', { name: 'Déduire d’un exemple JSON' }),
		)
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onChange).not.toHaveBeenCalled()
		expect(screen.queryByLabelText('Exemple de corps reçu')).toBeNull()
		expect(
			within(document.body).getByRole('button', { name: 'Ajouter un champ' }),
		).toBeDefined()
	})
})

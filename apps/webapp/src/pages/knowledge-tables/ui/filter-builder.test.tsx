import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	FilterDraft,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'
import { FilterBuilder } from '#/pages/knowledge-tables/ui/filter-builder'

const name: TableColumn = {
	id: 'name',
	name: 'Nom',
	kind: 'text',
	config: {},
	is_primary: true,
	position: 0,
}
const amount: TableColumn = {
	...name,
	id: 'amount',
	name: 'Montant',
	kind: 'currency',
	is_primary: false,
	position: 1,
}
const status: TableColumn = {
	...name,
	id: 'status',
	name: 'Statut',
	kind: 'single_select',
	is_primary: false,
	position: 2,
	config: {
		options: [
			{ id: 'todo', label: 'À faire', color: 'gray' },
			{ id: 'doing', label: 'En cours', color: 'blue' },
		],
	},
}
const done: TableColumn = {
	...name,
	id: 'done',
	name: 'Traité',
	kind: 'checkbox',
	is_primary: false,
	position: 3,
}

function renderBuilder(filters: FilterDraft[], onChange = vi.fn()) {
	render(
		<FilterBuilder
			columns={[name, amount, status, done]}
			filters={filters}
			choicesFor={() => []}
			onChange={onChange}
		/>,
	)
	return onChange
}

function operatorOptions() {
	return within(screen.getByRole('combobox', { name: 'Opérateur' }))
		.getAllByRole('option')
		.map((option) => option.textContent)
}

describe('FilterBuilder', () => {
	it('offers the text operators on a text column', () => {
		renderBuilder([
			{ key: 'f1', column_id: 'name', operator: 'contains', value: '' },
		])
		expect(operatorOptions()).toEqual([
			'contient',
			'est',
			'est vide',
			'n’est pas vide',
		])
	})

	it('offers the comparisons on an amount', () => {
		renderBuilder([
			{ key: 'f1', column_id: 'amount', operator: 'gt', value: '' },
		])
		expect(operatorOptions()).toEqual([
			'=',
			'≠',
			'<',
			'≤',
			'>',
			'≥',
			'est vide',
			'n’est pas vide',
		])
	})

	it('offers checked and unchecked on a checkbox, without value', () => {
		renderBuilder([
			{ key: 'f1', column_id: 'done', operator: 'checked', value: null },
		])
		expect(operatorOptions()).toEqual(['cochée', 'non cochée'])
		expect(screen.queryByRole('textbox', { name: 'Valeur' })).toBeNull()
	})

	it('offers the options of a single choice as values', () => {
		renderBuilder([
			{ key: 'f1', column_id: 'status', operator: 'is', value: '' },
		])
		expect(operatorOptions()).toEqual(['est', 'n’est pas', 'parmi', 'est vide'])
		const values = within(screen.getByRole('combobox', { name: 'Valeur' }))
			.getAllByRole('option')
			.map((option) => option.textContent)
		expect(values).toEqual(['Choisir…', 'À faire', 'En cours'])
	})

	it('resets the operator when the column changes', async () => {
		const onChange = renderBuilder([
			{ key: 'f1', column_id: 'name', operator: 'contains', value: 'abc' },
		])
		await userEvent.selectOptions(
			screen.getByRole('combobox', { name: 'Colonne' }),
			'amount',
		)
		expect(onChange).toHaveBeenCalledWith([
			{ key: 'f1', column_id: 'amount', operator: 'eq', value: '' },
		])
	})

	it('adds a filter on the first column', async () => {
		const onChange = renderBuilder([])
		await userEvent.click(
			screen.getByRole('button', { name: 'Ajouter un filtre' }),
		)
		expect(onChange).toHaveBeenCalledWith([
			expect.objectContaining({
				column_id: 'name',
				operator: 'contains',
				value: '',
			}),
		])
	})

	it('removes a filter', async () => {
		const onChange = renderBuilder([
			{ key: 'f1', column_id: 'name', operator: 'is_empty', value: null },
		])
		await userEvent.click(
			screen.getByRole('button', { name: 'Retirer le filtre' }),
		)
		expect(onChange).toHaveBeenCalledWith([])
	})
})

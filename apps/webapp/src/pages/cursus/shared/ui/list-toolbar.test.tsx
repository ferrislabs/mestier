import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { FEMININE_ARCHIVE_OPTIONS } from '#/pages/cursus/shared/lib/list-filters'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'
import {
	chooseOption,
	installRadixPolyfills,
} from '#/pages/support/feature/support-test-kit'

beforeAll(installRadixPolyfills)

function renderToolbar() {
	const handlers = {
		onSearch: vi.fn(),
		onStatus: vi.fn(),
		onSelect: vi.fn(),
	}
	render(
		<ListToolbar
			search=""
			searchPlaceholder="Rechercher une promotion"
			onSearch={handlers.onSearch}
			status={['active']}
			statusOptions={FEMININE_ARCHIVE_OPTIONS}
			statusLabel="Statut"
			onStatus={handlers.onStatus}
			select={{
				value: 'all',
				label: 'Filtrer par formation',
				options: [
					{ value: 'all', label: 'Toutes les formations' },
					{ value: 'training-1', label: 'CAP Menuisier' },
				],
				onChange: handlers.onSelect,
			}}
		/>,
	)
	return handlers
}

describe('ListToolbar', () => {
	it('reports the typed search', async () => {
		const user = userEvent.setup()
		const handlers = renderToolbar()

		await user.click(screen.getByPlaceholderText('Rechercher une promotion'))
		await user.paste('CAP')

		expect(handlers.onSearch).toHaveBeenCalledWith('CAP')
	})

	it('reports the chosen status', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		const handlers = renderToolbar()

		await user.click(screen.getByRole('button', { name: 'Statut' }))
		await user.click(
			screen.getByRole('menuitemcheckbox', { name: 'Archivées' }),
		)

		expect(handlers.onStatus).toHaveBeenCalledWith(['active', 'archived'])
	})

	it('reports the chosen select option', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		const handlers = renderToolbar()

		await chooseOption(
			user,
			screen.getByRole('combobox', { name: 'Filtrer par formation' }),
			'CAP Menuisier',
		)

		expect(handlers.onSelect).toHaveBeenCalledWith('training-1')
	})

	it('narrows the select options with its search box', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderToolbar()

		await user.click(
			screen.getByRole('combobox', { name: 'Filtrer par formation' }),
		)
		await user.type(await screen.findByPlaceholderText('Rechercher…'), 'menu')

		expect(screen.getByRole('option', { name: 'CAP Menuisier' })).toBeTruthy()
		expect(
			screen.queryByRole('option', { name: 'Toutes les formations' }),
		).toBeNull()
	})

	it('has no select when none is given', () => {
		render(
			<ListToolbar
				search=""
				searchPlaceholder="Rechercher"
				onSearch={vi.fn()}
				status={['active']}
				statusOptions={FEMININE_ARCHIVE_OPTIONS}
				statusLabel="Statut"
				onStatus={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('combobox')).toBeNull()
	})
})

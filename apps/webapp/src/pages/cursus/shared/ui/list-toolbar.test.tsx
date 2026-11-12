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
			status="active"
			statusOptions={FEMININE_ARCHIVE_OPTIONS}
			statusLabel="Filtrer par statut"
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

	it('reports the chosen status', async () => {
		const user = userEvent.setup()
		const handlers = renderToolbar()

		await user.click(screen.getByRole('radio', { name: 'Archivées' }))

		expect(handlers.onStatus).toHaveBeenCalledWith('archived')
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

	it('has no select when none is given', () => {
		render(
			<ListToolbar
				search=""
				searchPlaceholder="Rechercher"
				onSearch={vi.fn()}
				status="active"
				statusOptions={FEMININE_ARCHIVE_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('combobox')).toBeNull()
	})
})

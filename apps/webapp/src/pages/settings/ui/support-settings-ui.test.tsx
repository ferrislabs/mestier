import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	SupportSettingsUI,
	type SupportSettingsUIProps,
} from '#/pages/settings/ui/support-settings-ui'

const READ_ONLY_NOTE =
	'Seuls les membres autorisés à traiter les tickets peuvent modifier ce réglage.'

function renderUI(overrides: Partial<SupportSettingsUIProps> = {}) {
	const props: SupportSettingsUIProps = {
		isLoading: false,
		value: 'chantier\nmarque',
		canManage: true,
		isPending: false,
		formError: null,
		saveError: null,
		onChange: vi.fn(),
		onSubmit: vi.fn(),
		...overrides,
	}
	render(<SupportSettingsUI {...props} />)
	return props
}

function field() {
	return screen.getByLabelText(
		'Clés des champs personnalisés',
	) as HTMLTextAreaElement
}

describe('SupportSettingsUI', () => {
	it('shows only a spinner while loading', () => {
		renderUI({ isLoading: true })

		expect(screen.getByText('Chargement…')).toBeDefined()
		expect(screen.queryByLabelText('Clés des champs personnalisés')).toBeNull()
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
	})

	it('titles the card and explains what the keys do', () => {
		renderUI()

		expect(
			screen.getByRole('heading', { name: 'Champs affichés' }),
		).toBeDefined()
		expect(
			screen.getByText(/champs personnalisés.*cartes.*colonnes/i),
		).toBeDefined()
	})

	it('shows the value in a labelled textarea with its helper text', () => {
		renderUI({ value: 'chantier\nmarque' })

		expect(field().tagName).toBe('TEXTAREA')
		expect(field().value).toBe('chantier\nmarque')
		expect(
			screen.getByText(
				'Une clé par ligne, huit au plus. L’ordre des lignes est l’ordre d’affichage.',
			),
		).toBeDefined()
	})

	it('reports every edit through onChange', () => {
		const props = renderUI({ value: '' })

		fireEvent.change(field(), { target: { value: 'chantier' } })

		expect(props.onChange).toHaveBeenCalledWith('chantier')
	})

	it('is editable and offers the save button to a caller who can manage', () => {
		renderUI({ canManage: true })

		expect(field().readOnly).toBe(false)
		expect(screen.getByRole('button', { name: 'Enregistrer' })).toBeDefined()
		expect(screen.queryByText(READ_ONLY_NOTE)).toBeNull()
	})

	it('is read-only, without a save button, for a caller who cannot manage', () => {
		renderUI({ canManage: false })

		expect(field().readOnly).toBe(true)
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
		expect(screen.getByText(READ_ONLY_NOTE)).toBeDefined()
	})

	it('disables the save button while the request is pending', () => {
		renderUI({ isPending: true })

		const button = screen.getByRole('button', {
			name: 'Enregistrer',
		}) as HTMLButtonElement
		expect(button.disabled).toBe(true)
	})

	it('keeps the save button enabled when idle', () => {
		renderUI({ isPending: false })

		const button = screen.getByRole('button', {
			name: 'Enregistrer',
		}) as HTMLButtonElement
		expect(button.disabled).toBe(false)
	})

	it('calls onSubmit once when the save button is clicked', async () => {
		const user = userEvent.setup()
		const props = renderUI()

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(props.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('shows the form error and the save error as destructive text', () => {
		renderUI({
			formError: 'Huit clés au plus.',
			saveError: 'Le serveur a refusé la liste.',
		})

		const formError = screen.getByText('Huit clés au plus.')
		const saveError = screen.getByText('Le serveur a refusé la liste.')
		expect(formError.className).toContain('text-destructive')
		expect(saveError.className).toContain('text-destructive')
	})

	it('shows no error text when there is none', () => {
		renderUI()

		expect(document.querySelector('.text-destructive')).toBeNull()
	})
})

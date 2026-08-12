import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { InviteMemberSheet } from '#/pages/hr/ui/invite-member-sheet'
import { renderWithRouter } from '#/test/render-with-router'

afterEach(() => {
	vi.unstubAllGlobals()
})

function baseProps() {
	return {
		open: true,
		memberName: 'Alix Nova',
		token: null as string | null,
		isGenerating: false,
		error: null as string | null,
		onOpenChange: vi.fn(),
		onGenerate: vi.fn(),
	}
}

describe('InviteMemberSheet — before generation', () => {
	it('offers a "Générer le lien" button and calls onGenerate', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<InviteMemberSheet {...props} />)

		await user.click(screen.getByRole('button', { name: /Générer le lien/ }))

		expect(props.onGenerate).toHaveBeenCalledOnce()
	})

	it('shows no link input until a token exists', async () => {
		await renderWithRouter(<InviteMemberSheet {...baseProps()} />)

		expect(screen.queryByLabelText('Lien d’invitation')).toBeNull()
	})
})

describe('InviteMemberSheet — after generation', () => {
	it('shows the link built from the token, and copies it on click', async () => {
		const user = userEvent.setup()
		const writeText = vi.fn().mockResolvedValue(undefined)
		vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } })

		await renderWithRouter(
			<InviteMemberSheet {...baseProps()} token="abc123" />,
		)

		const input = screen.getByLabelText('Lien d’invitation') as HTMLInputElement
		expect(input.value).toContain('/invite/abc123')

		await user.click(screen.getByRole('button', { name: 'Copier' }))

		expect(writeText).toHaveBeenCalledWith(input.value)
		expect(screen.getByText('Copié')).toBeDefined()
	})
})

describe('InviteMemberSheet — error', () => {
	it('shows the error message when generation fails', async () => {
		await renderWithRouter(
			<InviteMemberSheet {...baseProps()} error="La génération a échoué." />,
		)

		expect(screen.getByText('La génération a échoué.')).toBeDefined()
	})
})

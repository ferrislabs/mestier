import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { CredentialSecretReveal } from '#/pages/automation/ui/credential-secret-reveal'

describe('CredentialSecretReveal', () => {
	it('shows the secret and warns it will not be shown again', () => {
		render(<CredentialSecretReveal secret="s3cr3t-value" onDone={vi.fn()} />)

		expect((screen.getByLabelText('Secret') as HTMLInputElement).value).toBe(
			's3cr3t-value',
		)
		expect(screen.getByText(/ne sera plus jamais affiché/i)).toBeDefined()
	})

	it('copies the secret to the clipboard', async () => {
		const user = userEvent.setup()
		const writeText = vi.fn().mockResolvedValue(undefined)
		vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } })

		render(<CredentialSecretReveal secret="s3cr3t-value" onDone={vi.fn()} />)

		await user.click(screen.getByRole('button', { name: /Copier/ }))

		expect(writeText).toHaveBeenCalledWith('s3cr3t-value')
	})

	it('calls onDone when the user continues', async () => {
		const user = userEvent.setup()
		const onDone = vi.fn()
		render(<CredentialSecretReveal secret="s3cr3t-value" onDone={onDone} />)

		await user.click(screen.getByRole('button', { name: /Continuer/ }))

		expect(onDone).toHaveBeenCalledTimes(1)
	})
})

import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { DeveloperModeOnly } from '#/components/developer-mode-only'
import { DeveloperSection } from '#/pages/settings/sections/developer-section'

describe('DeveloperSection', () => {
	beforeEach(() => {
		window.localStorage.removeItem('mestier.developerMode')
	})

	afterEach(() => {
		window.localStorage.removeItem('mestier.developerMode')
	})

	it('starts unchecked and reveals a developer-only control once toggled on', async () => {
		const user = userEvent.setup()
		render(
			<>
				<DeveloperSection />
				<DeveloperModeOnly>
					<p>id: abc-123</p>
				</DeveloperModeOnly>
			</>,
		)

		const toggle = screen.getByRole('switch', { name: /identifiants/i })
		expect(toggle).toHaveProperty('ariaChecked', 'false')
		expect(screen.queryByText('id: abc-123')).toBeNull()

		await user.click(toggle)

		expect(toggle).toHaveProperty('ariaChecked', 'true')
		expect(screen.getByText('id: abc-123')).toBeDefined()
	})
})

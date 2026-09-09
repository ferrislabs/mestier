import { act, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { DeveloperModeOnly } from '#/components/developer-mode-only'
import { setDeveloperMode } from '#/hooks/use-developer-mode'

describe('DeveloperModeOnly', () => {
	beforeEach(() => {
		window.localStorage.removeItem('mestier.developerMode')
	})

	afterEach(() => {
		window.localStorage.removeItem('mestier.developerMode')
	})

	it('hides its children when developer mode is off', () => {
		render(
			<DeveloperModeOnly>
				<p>id: abc-123</p>
			</DeveloperModeOnly>,
		)

		expect(screen.queryByText('id: abc-123')).toBeNull()
	})

	it('shows its children once developer mode is turned on', () => {
		render(
			<DeveloperModeOnly>
				<p>id: abc-123</p>
			</DeveloperModeOnly>,
		)

		act(() => setDeveloperMode(true))

		expect(screen.getByText('id: abc-123')).toBeDefined()
	})

	it('hides its children again once developer mode is turned back off', () => {
		render(
			<DeveloperModeOnly>
				<p>id: abc-123</p>
			</DeveloperModeOnly>,
		)

		act(() => setDeveloperMode(true))
		act(() => setDeveloperMode(false))

		expect(screen.queryByText('id: abc-123')).toBeNull()
	})
})

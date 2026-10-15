import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest'
import { FieldInterventionFeature } from '#/pages/maintenance/feature/field-intervention-feature'

// "Today" is what the date field starts on, so the suite pins it. Only `Date`
// is faked, and the clock still ticks, so `waitFor` keeps working.
beforeAll(() => {
	vi.useFakeTimers({
		now: new Date('2026-10-14T10:00:00'),
		shouldAdvanceTime: true,
		toFake: ['Date'],
	})
})

afterAll(() => {
	vi.useRealTimers()
})

const PROPOSE_PATH =
	'/api/v1/organizations/{organization_id}/field/tasks/{task_id}/intervention'

function renderButton(onPropose: (params: unknown) => unknown) {
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = {
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (method === 'post' && path === PROPOSE_PATH) {
							return onPropose(params)
						}
						throw new Error(`unmocked ${method} ${path}`)
					},
				},
			}
		},
	}
	const queryClient = new QueryClient({
		defaultOptions: { mutations: { retry: false } },
	})
	render(
		<QueryClientProvider client={queryClient}>
			<FieldInterventionFeature organizationId="org-1" taskId="task-1" />
		</QueryClientProvider>,
	)
}

describe('FieldInterventionFeature', () => {
	it('declares the visit done today with one confirmation', async () => {
		const onPropose = vi.fn(() => ({ data: {}, pagination: null }))
		const user = userEvent.setup()
		renderButton(onPropose)

		await user.click(screen.getByRole('button', { name: 'Intervention faite' }))
		const date = screen.getByLabelText(/date de l.intervention/i)
		expect((date as HTMLInputElement).value).toBe('2026-10-14')
		await user.click(screen.getByRole('button', { name: 'Envoyer' }))

		await waitFor(() => expect(onPropose).toHaveBeenCalledTimes(1))
		expect(onPropose).toHaveBeenCalledWith({
			path: { organization_id: 'org-1', task_id: 'task-1' },
			body: { done_on: '2026-10-14' },
		})
		expect(await screen.findByText(/14 oct\. envoyée/i)).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Envoyer' })).toBeNull()
	})

	it('sends the day the worker picked instead of today', async () => {
		const onPropose = vi.fn(() => ({ data: {}, pagination: null }))
		const user = userEvent.setup()
		renderButton(onPropose)

		await user.click(screen.getByRole('button', { name: 'Intervention faite' }))
		fireEvent.change(screen.getByLabelText(/date de l.intervention/i), {
			target: { value: '2026-10-12' },
		})
		await user.click(screen.getByRole('button', { name: 'Envoyer' }))

		await waitFor(() => expect(onPropose).toHaveBeenCalledTimes(1))
		expect(onPropose).toHaveBeenCalledWith(
			expect.objectContaining({ body: { done_on: '2026-10-12' } }),
		)
	})

	it('never offers a day after today', async () => {
		const user = userEvent.setup()
		renderButton(vi.fn())

		await user.click(screen.getByRole('button', { name: 'Intervention faite' }))

		const date = screen.getByLabelText(/date de l.intervention/i)
		expect(date.getAttribute('max')).toBe('2026-10-14')
	})

	it('keeps the form open and says why when the date is refused', async () => {
		const refused = Object.assign(new Error('outside the season'), {
			status: 409,
		})
		const user = userEvent.setup()
		renderButton(() => {
			throw refused
		})

		await user.click(screen.getByRole('button', { name: 'Intervention faite' }))
		await user.click(screen.getByRole('button', { name: 'Envoyer' }))

		const alert = await screen.findByRole('alert')
		expect(alert.textContent).toMatch(/saison/i)
		expect(screen.getByRole('button', { name: 'Envoyer' })).toBeTruthy()
	})

	it('closes the form without sending anything', async () => {
		const onPropose = vi.fn()
		const user = userEvent.setup()
		renderButton(onPropose)

		await user.click(screen.getByRole('button', { name: 'Intervention faite' }))
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(
			screen.getByRole('button', { name: 'Intervention faite' }),
		).toBeTruthy()
		expect(onPropose).not.toHaveBeenCalled()
	})
})

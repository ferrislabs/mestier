import { QueryClient } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { DeleteDoeFeature } from '#/pages/doe/feature/delete-doe-feature'
import { DoeEditorFeature } from '#/pages/doe/feature/doe-editor-feature'
import type { Doe } from '#/pages/doe/types'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

const DOE_PATH = '/api/v1/organizations/{organization_id}/does/{doe_id}'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

function installFakeApi(
	onDelete: (params: unknown) => unknown,
	permissions: string[] = ['MANAGE_PLANNING'],
) {
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						return { data: [], pagination: null }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (method === 'delete' && path === DOE_PATH) {
							return onDelete(params)
						}
						throw new Error(`unmocked ${method} ${path}`)
					},
				},
			}
		},
	}
}

function wrap(ui: React.ReactElement, permissions = ['MANAGE_PLANNING']) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	const wrapped = wrapWithPermissions(ui, { permissions, queryClient })
	installFakeApi(() => undefined, permissions)
	return wrapped
}

describe('DeleteDoeFeature', () => {
	it('calls the API only after the confirmation', async () => {
		const onDelete = vi.fn(() => undefined)
		const onDeleted = vi.fn()
		const user = userEvent.setup()
		const ui = wrap(
			<DeleteDoeFeature
				open
				organizationId="org-1"
				doeId="d1"
				title="DOE Duval"
				onOpenChange={() => {}}
				onDeleted={onDeleted}
			/>,
		)
		installFakeApi(onDelete)
		render(ui)

		expect(screen.getByText('Supprimer ce DOE ?')).toBeTruthy()
		expect(onDelete).not.toHaveBeenCalled()

		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		await waitFor(() => expect(onDelete).toHaveBeenCalledTimes(1))
		const params = onDelete.mock.calls[0] as unknown as [
			{ path: { organization_id: string; doe_id: string } },
		]
		expect(params[0].path).toEqual({ organization_id: 'org-1', doe_id: 'd1' })
		await waitFor(() => expect(onDeleted).toHaveBeenCalledTimes(1))
	})

	it('shows the API error inside the dialog', async () => {
		const onDelete = vi.fn(() => {
			throw new Error('Introuvable')
		})
		const user = userEvent.setup()
		const ui = wrap(
			<DeleteDoeFeature
				open
				organizationId="org-1"
				doeId="d1"
				title="DOE Duval"
				onOpenChange={() => {}}
				onDeleted={() => {}}
			/>,
		)
		installFakeApi(onDelete)
		render(ui)

		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)
		await waitFor(() => expect(screen.getByRole('alert')).toBeTruthy())
		expect(screen.getByRole('alert').textContent).toMatch(/Introuvable/)
	})
})

const DOE: Doe = {
	id: 'd1',
	project_id: 'p1',
	title: 'Avenue du 8 mai',
	issued_on: '2026-10-01',
	revision: 'A',
	parties: [],
	sections: [],
	created_at: '2026-10-01T00:00:00Z',
	updated_at: '2026-10-01T00:00:00Z',
}

function editor() {
	return (
		<DoeEditorFeature
			doe={DOE}
			organizationId="org-1"
			organizationSlug="organisation"
			projectId="p1"
			projectName="Jardin"
		/>
	)
}

describe('DoeEditorFeature deletion', () => {
	it('goes back to the project page once deleted', async () => {
		const onDelete = vi.fn(() => undefined)
		const user = userEvent.setup()
		const ui = wrap(editor())
		installFakeApi(onDelete)
		const { router } = await renderWithRouter(ui)

		await user.click(screen.getByRole('button', { name: 'Supprimer le DOE' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		await waitFor(() =>
			expect(router.state.location.pathname).toBe(
				'/o/organisation/planification/projects/p1',
			),
		)
		expect(onDelete).toHaveBeenCalledTimes(1)
	})

	it('hides the action without the planning permission', async () => {
		const ui = wrap(editor(), [])
		await renderWithRouter(ui)
		expect(
			screen.queryByRole('button', { name: 'Supprimer le DOE' }),
		).toBeNull()
	})
})

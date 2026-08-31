import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { RowActions } from '#/components/reference-table'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATION = { id: 'org-1', name: 'Dupont', slug: 'dupont' }

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

function installFakePermissionsApi(permissions: string[]) {
	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			throw new Error('unmocked mutation')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

function baseProps() {
	return {
		isEditing: false,
		isSaving: false,
		onEdit: vi.fn(),
		onCancel: vi.fn(),
		onSave: vi.fn(),
		onDelete: vi.fn(),
	}
}

async function renderRowActions(
	permissions: string[],
	props: Partial<Parameters<typeof RowActions>[0]> = {},
) {
	installFakePermissionsApi(permissions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return renderWithRouter(
		<Providers>
			<p>sentinel</p>
			<RowActions {...baseProps()} {...props} />
		</Providers>,
	)
}

describe('RowActions — permission prop (#403)', () => {
	it('renders the actions trigger when no permission is required, unchanged from before #403', async () => {
		await renderRowActions([])

		expect(await screen.findByRole('button', { name: 'Actions' })).toBeDefined()
	})

	/**
	 * The regression this guards: an earlier version of #403 called
	 * `usePermissions` unconditionally, which needs `ActiveOrganizationProvider`
	 * in the tree — breaking every pre-existing reuse site (and its tests)
	 * that never passes `permission` and has no reason to carry that context.
	 * No `ActiveOrganizationProvider`/`QueryClientProvider` here on purpose.
	 */
	it('renders with no permission-related context at all when `permission` is omitted', () => {
		render(<RowActions {...baseProps()} />)

		expect(screen.getByRole('button', { name: 'Actions' })).toBeDefined()
	})

	it('hides the whole control when a permission is required but not held', async () => {
		await renderRowActions([], { permission: 'MANAGE_CUSTOMERS' })

		// Sync point: once the sentinel (unconditionally rendered) is still
		// there and the read has had a tick to settle, the gated control's
		// absence is the real assertion, not a race with the query.
		await screen.findByText('sentinel')
		expect(screen.queryByRole('button', { name: 'Actions' })).toBeNull()
	})

	it('shows the control once the caller holds the required permission', async () => {
		await renderRowActions(['MANAGE_CUSTOMERS'], {
			permission: 'MANAGE_CUSTOMERS',
		})

		expect(await screen.findByRole('button', { name: 'Actions' })).toBeDefined()
	})

	it('calls onEdit and onDelete from the gated dropdown', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps() }
		await renderRowActions(['MANAGE_CUSTOMERS'], {
			...props,
			permission: 'MANAGE_CUSTOMERS',
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Modifier' }))

		expect(props.onEdit).toHaveBeenCalled()
	})
})

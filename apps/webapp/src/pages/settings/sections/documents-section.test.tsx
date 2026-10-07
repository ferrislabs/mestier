import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import {
	DocumentsSection,
	isValidAccent,
} from '#/pages/settings/sections/documents-section'
import { seedPermissionsCache } from '#/test/with-permissions'

const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/doe-settings'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier',
	slug: 'atelier',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
	field_clock_enabled: false,
	vat_on_debits: false,
	missing_legal_identity_fields: [],
}

function installFakeApi(onPut: (params: unknown) => unknown) {
	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === SETTINGS_PATH) {
							return {
								data: { accent: '#0f3d36', logo_key: null },
								pagination: null,
							}
						}
						if (path === MY_PERMISSIONS_PATH) {
							return {
								data: { permissions: ALL_PERMISSIONS },
								pagination: null,
							}
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (method === 'put' && path === SETTINGS_PATH) return onPut(params)
						throw new Error(`unmocked ${method} ${path}`)
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake
	;(window as any).tanstackApi = fakeApi
}

function renderSection(onPut: (params: unknown) => unknown = vi.fn()) {
	installFakeApi(onPut)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCache(queryClient, { permissions: ALL_PERMISSIONS })
	return render(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<DocumentsSection />
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('isValidAccent', () => {
	it('accepts #rrggbb only', () => {
		expect(isValidAccent('#0f3d36')).toBe(true)
		expect(isValidAccent('#ABCDEF')).toBe(true)
		expect(isValidAccent('0f3d36')).toBe(false)
		expect(isValidAccent('#0f3d3')).toBe(false)
		expect(isValidAccent('#0f3d36a')).toBe(false)
		expect(isValidAccent('#gggggg')).toBe(false)
		expect(isValidAccent('')).toBe(false)
	})
})

describe('DocumentsSection', () => {
	it('flags an invalid hex and keeps save disabled', async () => {
		const user = userEvent.setup()
		renderSection()
		const input = await screen.findByLabelText('Couleur personnalisée')

		await user.clear(input)
		await user.type(input, '#12')

		expect(screen.getByRole('alert').textContent).toMatch(/#rrggbb/)
		expect(
			(
				screen.getByRole('button', {
					name: /enregistrer/i,
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('saves the swatch picked', async () => {
		const onPut = vi.fn(() => ({ data: { accent: '#6b1460', logo_key: null } }))
		const user = userEvent.setup()
		renderSection(onPut)

		await user.click(
			await screen.findByRole('button', { name: 'Couleur #6b1460' }),
		)
		await user.click(screen.getByRole('button', { name: /enregistrer/i }))

		await waitFor(() => expect(onPut).toHaveBeenCalledTimes(1))
		const params = onPut.mock.calls[0] as unknown as [
			{ body: { accent: string; logo_key: string | null } },
		]
		expect(params[0].body).toEqual({ accent: '#6b1460', logo_key: null })
	})

	it('saves a free hex typed in', async () => {
		const onPut = vi.fn(() => ({ data: { accent: '#123abc', logo_key: null } }))
		const user = userEvent.setup()
		renderSection(onPut)
		const input = await screen.findByLabelText('Couleur personnalisée')

		await user.clear(input)
		await user.type(input, '#123ABC')
		await user.click(screen.getByRole('button', { name: /enregistrer/i }))

		await waitFor(() => expect(onPut).toHaveBeenCalledTimes(1))
		const params = onPut.mock.calls[0] as unknown as [
			{ body: { accent: string } },
		]
		expect(params[0].body.accent).toBe('#123abc')
	})
})

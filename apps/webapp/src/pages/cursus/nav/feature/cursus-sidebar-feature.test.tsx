import { screen, waitFor } from '@testing-library/react'
import { beforeAll, describe, expect, it } from 'vitest'
import { CursusSidebarFeature } from '#/pages/cursus/nav/feature/cursus-sidebar-feature'
import {
	createQueryClient,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'
import { renderWithRouter } from '#/test/render-with-router'

const BASE = '/api/v1/organizations/{organization_id}/cursus'

beforeAll(installRadixPolyfills)

describe('CursusSidebarFeature', () => {
	it('counts company and school tutors together', {
		timeout: 15_000,
	}, async () => {
		const api = installFakeTanstackApi()
		api.mockGet(`${BASE}/participants`, (params) => {
			const kind = (params as { query: { kind: string } }).query.kind
			return page([] as never, kind === 'company_tutor' ? 7 : 12)
		})
		api.mockGet(`${BASE}/school-tutors`, () => page([] as never, 3))
		await renderWithRouter(
			<Providers queryClient={createQueryClient(['VIEW_CURSUS'])}>
				<CursusSidebarFeature />
			</Providers>,
			'/o/atelier-bois/cursus/tuteurs',
		)

		await waitFor(() =>
			expect(
				screen.getByRole('link', { name: /Tuteurs/ }).textContent,
			).toContain('10'),
		)
	})
})

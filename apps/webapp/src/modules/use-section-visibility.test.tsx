import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { MODULES } from '#/modules/registry'
import { isTargetVisible } from '#/modules/use-section-visibility'
import { useVisibleModules } from '#/modules/use-visible-modules'
import {
	installFakeTanstackApi,
	ORGANIZATION,
} from '#/pages/support/feature/support-test-kit'

const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const SPACES_PATH = '/api/v1/cursus/me/spaces'

const SPACE = {
	organization_id: ORGANIZATION.id,
	slug: ORGANIZATION.slug,
	name: ORGANIZATION.name,
	is_member: true,
}

function cursusSections() {
	const module = MODULES.find((candidate) => candidate.id === 'cursus')
	if (!module) throw new Error('registry: cursus module missing')
	return module.sections
}

describe('Mes trinômes section', () => {
	it('is gated on being a participant, never on a permission bit', () => {
		const section = cursusSections().find((s) => s.id === 'mes-trinomes')

		expect(section?.to).toBe('/cursus/mes-trinomes')
		expect(section?.visibleWhen).toBe('cursus-participant')
		expect(section?.requiredPermission).toBeUndefined()
	})

	it('is visible to a participant without any bit', () => {
		expect(
			isTargetVisible(
				{ visibleWhen: 'cursus-participant' },
				{ granted: [], loaded: true, isCursusParticipant: true },
			),
		).toBe(true)
	})

	it('is hidden from a non participant even holding VIEW_CURSUS', () => {
		expect(
			isTargetVisible(
				{ visibleWhen: 'cursus-participant' },
				{ granted: ['VIEW_CURSUS'], loaded: true, isCursusParticipant: false },
			),
		).toBe(false)
	})

	it('keeps the permission rule for the other sections', () => {
		const target = { requiredPermission: 'VIEW_CURSUS' as const }

		expect(
			isTargetVisible(target, {
				granted: ['VIEW_CURSUS'],
				loaded: true,
				isCursusParticipant: false,
			}),
		).toBe(true)
		expect(
			isTargetVisible(target, {
				granted: [],
				loaded: true,
				isCursusParticipant: true,
			}),
		).toBe(false)
	})
})

describe('useVisibleModules with Cursus', () => {
	function setup(spaces: unknown[]) {
		const api = installFakeTanstackApi()
		api.mockGet(PERMISSIONS_PATH, () => ({ data: { permissions: [] } }))
		api.mockGet(SPACES_PATH, () => ({ data: spaces }))
		const queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		})
		const wrapper = ({ children }: { children: ReactNode }) => (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
		return renderHook(() => useVisibleModules(), { wrapper })
	}

	it('shows Cursus to a participant without the Cursus bit', async () => {
		const { result } = setup([SPACE])

		await waitFor(() =>
			expect(result.current.some((module) => module.id === 'cursus')).toBe(
				true,
			),
		)
	})

	it('hides Cursus from someone with no bit and no participant seat', async () => {
		const { result } = setup([])

		await waitFor(() => expect(result.current.length).toBeGreaterThan(0))
		await new Promise((resolve) => setTimeout(resolve, 50))
		expect(result.current.some((module) => module.id === 'cursus')).toBe(false)
	})
})

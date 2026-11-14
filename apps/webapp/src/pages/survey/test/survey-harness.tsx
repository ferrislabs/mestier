import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	type FakeApi,
	installFakeTanstackApi,
} from '#/pages/automation/test/fake-tanstack-api'
import type { Campaign, Survey } from '#/pages/survey/types'
import { renderWithRouter } from '#/test/render-with-router'

export const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

export const PATHS = {
	permissions: '/api/v1/organizations/{organization_id}/members/me/permissions',
	surveys: '/api/v1/organizations/{organization_id}/surveys',
	survey: '/api/v1/organizations/{organization_id}/surveys/{survey_id}',
	campaigns: '/api/v1/organizations/{organization_id}/survey-campaigns',
}

export function survey(patch: Partial<Survey> = {}): Survey {
	return {
		id: 'survey-1',
		organization_id: ORGANIZATION.id,
		title: 'Bilan de session',
		description: null,
		visibility: 'PUBLIC',
		status: 'DRAFT',
		allow_multiple_responses: false,
		response_cap: null,
		campaign_id: null,
		metadata: {},
		graph: null,
		public_slug: null,
		opened_at: null,
		closed_at: null,
		...patch,
	}
}

export function campaign(patch: Partial<Campaign> = {}): Campaign {
	return {
		id: 'campaign-1',
		organization_id: ORGANIZATION.id,
		name: 'Session de mai',
		starts_at: '2026-05-01T08:00:00Z',
		ends_at: null,
		metadata: {},
		status: 'SCHEDULED',
		...patch,
	}
}

export function installSurveyApi({
	permissions,
	surveys = [],
	campaigns = [],
}: {
	permissions: string[]
	surveys?: Survey[]
	campaigns?: Campaign[]
}): FakeApi {
	const api = installFakeTanstackApi()
	api.mockGet(PATHS.permissions, () => ({
		data: { permissions },
		pagination: null,
	}))
	api.mockGet(PATHS.surveys, () => ({ data: surveys, pagination: null }))
	api.mockGet(PATHS.campaigns, () => ({ data: campaigns, pagination: null }))
	api.mockGet(PATHS.survey, (params) => {
		const id = (params as { path: { survey_id: string } }).path.survey_id
		const found = surveys.find((item) => item.id === id)
		if (!found) throw new Error(`unknown survey ${id}`)
		return { data: found, pagination: null }
	})
	return api
}

export function renderSurveyPage(ui: ReactNode) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<TooltipProvider>{ui}</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
		`/o/${ORGANIZATION.slug}/questionnaires`,
	)
}

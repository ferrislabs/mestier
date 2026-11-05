import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { Project } from '#/hooks/use-projects'
import { EMPTY_EXTRAS } from '#/pages/projects/lib/model'
import {
	ProjectDetailUI,
	type ProjectDetailUIProps,
} from '#/pages/projects/ui/project-detail-ui'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

// jsdom has neither `ResizeObserver` nor pointer capture, which the Radix
// primitives on this page probe defensively.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false

const PROJECT: Project = {
	id: 'project-1',
	organization_id: 'org-1',
	name: 'Toiture Duval',
	is_internal: false,
	customer_id: null,
	customer_context_id: null,
	quote_id: null,
	archived_at: null,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const PROPS: ProjectDetailUIProps = {
	organizationSlug: 'acme',
	project: PROJECT,
	status: 'active',
	customer: null,
	extras: EMPTY_EXTRAS,
	owner: undefined,
	people: [],
	missions: [],
	doe: null,
	isDoeLoading: false,
	canOpenDoe: false,
	isBuildingTemplate: false,
	onEdit: () => {},
	onToggleArchive: () => {},
	onSaveAsTemplate: () => {},
	onMissionStatusChange: () => {},
	billingSummary: null,
	isBillingSummaryLoading: false,
	period: { from: '2026-09-01', to: '2026-09-30' },
	onPeriodChange: () => {},
	costRow: null,
	isCostLoading: false,
	invoices: [],
	isInvoicesLoading: false,
	supplierCosts: null,
	isSupplierCostsLoading: false,
	hasCustomer: false,
	hasPinnedAddress: false,
	hasQuote: false,
	mode: 'FREE_AMOUNT',
	onModeChange: () => {},
	percentageInput: '',
	onPercentageInputChange: () => {},
	percentagePreviewCents: null,
	freeAmountInput: '',
	onFreeAmountInputChange: () => {},
	freeLabel: '',
	onFreeLabelChange: () => {},
	freeCustomerContexts: [],
	isFreeCustomerContextsLoading: false,
	freeCustomerContextId: '',
	onFreeCustomerContextIdChange: () => {},
	dueAt: '',
	onDueAtChange: () => {},
	notes: '',
	onNotesChange: () => {},
	isIssuing: false,
	error: null,
	onConfirm: () => {},
}

/**
 * The other half of #467's "focus on one project": getting there. The board
 * reads its filters from the search params, so a project's own page can hand
 * out a pre-filtered board as a plain link rather than sending the user to
 * an unfiltered board and a picker.
 */
describe('ProjectDetailUI — lien vers le tableau', () => {
	it('links to the board already filtered on this project', async () => {
		await renderWithRouter(wrapWithPermissions(<ProjectDetailUI {...PROPS} />))

		const link = screen.getByRole('link', { name: /Voir dans le tableau/ })

		expect(link.getAttribute('href')).toBe(
			'/o/acme/planification/board?project_id=project-1',
		)
	})
})

describe('ProjectDetailUI — carte DOE', () => {
	it('shows the revision when the project has a DOE', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectDetailUI
					{...PROPS}
					canOpenDoe
					doe={{
						revision: 'B',
						enabledSections: 2,
						supplies: 3,
						photos: 1,
						plans: 0,
					}}
				/>,
			),
		)

		const link = screen.getByRole('link', { name: 'Ouvrir le DOE · indice B' })

		expect(link.getAttribute('href')).toBe('/o/acme/projet/project-1/doe')
	})

	it('still offers to create the DOE when there is none yet', async () => {
		await renderWithRouter(
			wrapWithPermissions(<ProjectDetailUI {...PROPS} canOpenDoe />),
		)

		expect(screen.getByRole('link', { name: 'Créer le DOE' })).toBeDefined()
	})

	it('hides the link without the planning permission', async () => {
		await renderWithRouter(wrapWithPermissions(<ProjectDetailUI {...PROPS} />))

		expect(screen.queryByRole('link', { name: 'Créer le DOE' })).toBeNull()
	})
})

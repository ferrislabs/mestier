import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { computeBillingPipelineSteps } from '#/pages/quotes/lib/billing-pipeline'

function project(
	overrides: Partial<Schemas.ProjectResponse> = {},
): Schemas.ProjectResponse {
	return {
		id: 'project-1' as Schemas.ProjectId,
		organization_id: 'org-1' as Schemas.OrganizationId,
		name: 'Terrassement',
		is_internal: false,
		created_at: '2026-09-01T00:00:00Z',
		updated_at: '2026-09-01T00:00:00Z',
		...overrides,
	}
}

function invoice(
	overrides: Partial<Schemas.InvoiceResponse> = {},
): Schemas.InvoiceResponse {
	return {
		id: 'invoice-1' as Schemas.InvoiceId,
		organization_id: 'org-1' as Schemas.OrganizationId,
		customer_id: 'customer-1' as Schemas.CustomerId,
		customer_context_id: 'context-1' as Schemas.CustomerContextId,
		kind: 'STANDARD',
		status: 'ISSUED',
		lines: [],
		net_cents: 10_000,
		gross_cents: 10_000,
		vat_breakdown: [],
		created_at: '2026-09-01T00:00:00Z',
		updated_at: '2026-09-01T00:00:00Z',
		...overrides,
	}
}

describe('computeBillingPipelineSteps', () => {
	it('shows the quote in progress with everything else pending, before any project', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'SENT',
			project: undefined,
			billingSummary: undefined,
			invoices: undefined,
		})

		expect(steps.map((s) => [s.id, s.state])).toEqual([
			['quote', 'current'],
			['project', 'pending'],
			['deposit', 'pending'],
			['final', 'pending'],
			['paid', 'pending'],
		])
	})

	it('unlocks the project step once the quote is accepted', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: undefined,
			billingSummary: undefined,
			invoices: undefined,
		})

		expect(steps[0].state).toBe('done')
		expect(steps[1].state).toBe('current')
	})

	it('is a dead end for a declined quote — everything after is blocked, not pending', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'DECLINED',
			project: undefined,
			billingSummary: undefined,
			invoices: undefined,
		})

		expect(steps.map((s) => s.state)).toEqual([
			'stopped',
			'blocked',
			'blocked',
			'blocked',
			'blocked',
		])
	})

	it('marks the deposit step done from any non-draft deposit invoice, even with several', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: project(),
			billingSummary: undefined,
			invoices: [
				invoice({
					id: 'd1' as Schemas.InvoiceId,
					kind: 'DEPOSIT',
					status: 'ISSUED',
				}),
				invoice({
					id: 'd2' as Schemas.InvoiceId,
					kind: 'DEPOSIT',
					status: 'PAID',
				}),
			],
		})

		const deposit = steps.find((s) => s.id === 'deposit')
		expect(deposit?.state).toBe('done')
		expect(deposit?.label).toContain('Payée')
	})

	it('ignores draft and cancelled invoices for every step', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: project(),
			billingSummary: undefined,
			invoices: [
				invoice({ kind: 'DEPOSIT', status: 'DRAFT' }),
				invoice({ kind: 'FINAL', status: 'CANCELLED' }),
			],
		})

		expect(steps.find((s) => s.id === 'deposit')?.state).toBe('current')
		expect(steps.find((s) => s.id === 'final')?.state).toBe('current')
	})

	it('requires both full billing and full payment for the paid step, not remaining_cents alone', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: project(),
			billingSummary: {
				project_id: 'project-1' as Schemas.ProjectId,
				billed_cents: 10_000,
				remaining_cents: 0,
			},
			invoices: [invoice({ kind: 'FINAL', status: 'ISSUED' })],
		})

		expect(steps.find((s) => s.id === 'final')?.state).toBe('done')
		expect(steps.find((s) => s.id === 'paid')?.state).toBe('current')
	})

	it('reaches paid once billing is complete and every counted invoice is paid', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: project(),
			billingSummary: {
				project_id: 'project-1' as Schemas.ProjectId,
				billed_cents: 10_000,
				remaining_cents: 0,
			},
			invoices: [invoice({ kind: 'FINAL', status: 'PAID' })],
		})

		expect(steps.find((s) => s.id === 'paid')?.state).toBe('done')
	})

	it('nets out credit notes: a fully credited invoice does not count against being paid', () => {
		const steps = computeBillingPipelineSteps({
			quoteStatus: 'ACCEPTED',
			project: project(),
			billingSummary: {
				project_id: 'project-1' as Schemas.ProjectId,
				billed_cents: 0,
				remaining_cents: 0,
			},
			invoices: [
				invoice({ kind: 'FINAL', status: 'PAID' }),
				invoice({ kind: 'CREDIT_NOTE', status: 'ISSUED' }),
			],
		})

		// The credit note is excluded from the "every counted invoice paid" check.
		expect(steps.find((s) => s.id === 'paid')?.state).toBe('done')
	})
})

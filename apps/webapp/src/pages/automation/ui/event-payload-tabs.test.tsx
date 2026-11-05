import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { EventPayloadTabs } from '#/pages/automation/ui/event-payload-tabs'

function event(
	overrides: Partial<Schemas.EventDescriptorResponse>,
): Schemas.EventDescriptorResponse {
	return {
		label: 'Devis envoyé',
		name: 'quote.sent',
		payload_example: { quote: { id: 'a', status: 'SENT' }, from: 'DRAFT' },
		subject_kind: 'quote',
		version: 1,
		...overrides,
	}
}

describe('EventPayloadTabs', () => {
	it('shows the named TypeScript contract of a catalogued event', () => {
		const { container } = render(<EventPayloadTabs event={event({})} />)

		const source = container.querySelector('pre')?.textContent ?? ''
		expect(source).toContain('export interface QuoteSentPayload {')
		expect(source).toContain('  from: QuoteStatus')
		expect(source).toContain("type QuoteStatus = 'DRAFT'")
	})

	it('falls back to the type inferred from the example for another event', () => {
		const { container } = render(
			<EventPayloadTabs
				event={event({
					name: 'quote.archived',
					payload_example: { quote: { id: 'a' } },
				})}
			/>,
		)

		const source = container.querySelector('pre')?.textContent ?? ''
		expect(source).toContain('export interface QuoteArchivedPayload {')
		expect(source).toContain('    id: string')
		expect(source).not.toContain('QuoteStatus')
	})
})

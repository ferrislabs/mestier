import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { CollectionDetailFeature } from '#/pages/automation/feature/collection-detail-feature'
import { CollectionsListFeature } from '#/pages/automation/feature/collections-list-feature'
import { readCollections } from '#/pages/automation/feature/mocks/collections-store'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function Providers({ children }: { children: ReactNode }) {
	return (
		<QueryClientProvider client={new QueryClient()}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{children}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

describe('CollectionsListFeature', () => {
	it('lists the collections with their version and operation count', async () => {
		await renderWithRouter(
			<Providers>
				<CollectionsListFeature />
			</Providers>,
		)

		const first = readCollections()[0]
		if (!first) throw new Error('the mock store ships presets')
		const link = screen.getByRole('link', { name: first.title })
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/automatisation/collections/${first.id}`,
		)
		expect(screen.getAllByText(/opération/).length).toBeGreaterThan(0)
	})

	it('opens the import dialog', async () => {
		const user = userEvent.setup()
		await renderWithRouter(
			<Providers>
				<CollectionsListFeature />
			</Providers>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Importer une collection' }),
		)

		expect(
			await screen.findByRole('dialog', { name: 'Importer une collection' }),
		).toBeDefined()
		expect(screen.getByLabelText('URL de la spec')).toBeDefined()
	})
})

describe('CollectionDetailFeature', () => {
	it('expands an operation to show its contract', async () => {
		const user = userEvent.setup()
		const collection = readCollections()[0]
		if (!collection) throw new Error('the mock store ships presets')
		await renderWithRouter(
			<Providers>
				<CollectionDetailFeature collectionId={collection.id} />
			</Providers>,
		)

		const operation = collection.operations[0]
		if (!operation) throw new Error('presets have operations')
		await user.click(
			screen.getAllByRole('button', { expanded: false })[0] as HTMLElement,
		)

		expect(
			screen.getByRole('tab', { name: 'Contrat TypeScript' }),
		).toBeDefined()
		expect(screen.getAllByText(operation.path).length).toBeGreaterThan(0)
	})

	it('says so when the collection does not exist', async () => {
		await renderWithRouter(
			<Providers>
				<CollectionDetailFeature collectionId="nope" />
			</Providers>,
		)

		expect(screen.getByText('Collection introuvable')).toBeDefined()
		expect(
			screen
				.getByRole('link', { name: 'Voir les collections' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections')
	})

	it('links the breadcrumb back to the collections', async () => {
		const collection = readCollections()[0]
		if (!collection) throw new Error('the mock store ships presets')
		await renderWithRouter(
			<Providers>
				<CollectionDetailFeature collectionId={collection.id} />
			</Providers>,
		)

		expect(
			screen.getByRole('link', { name: 'Collections' }).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections')
	})
})

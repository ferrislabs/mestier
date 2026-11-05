import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Receipt, Users } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import type { AppModule } from '#/modules/types'
import type { HomeActivity } from '#/pages/home/types'
import { ActivityFeedUI } from '#/pages/home/ui/activity-feed-ui'
import { renderWithRouter } from '#/test/render-with-router'

const crm = { id: 'crm', label: 'CRM', icon: Users } as AppModule
const purchase = { id: 'purchase', label: 'Achats', icon: Receipt } as AppModule

const ACTIVITY: HomeActivity[] = [
	{
		id: 'a',
		title: 'Facture impayée',
		state: 'late',
		to: '/o/x/crm/invoices',
		module: crm,
	},
	{
		id: 'b',
		title: 'Facture fournisseur à valider',
		state: 'waiting',
		to: '/o/x/purchase',
		module: purchase,
	},
]

describe('ActivityFeedUI', () => {
	it('groups items by state and shows the empty message for empty groups', async () => {
		await renderWithRouter(
			<ActivityFeedUI activity={ACTIVITY} isLoading={false} />,
		)

		const now = screen.getByRole('region', { name: 'À traiter maintenant' })
		expect(within(now).getByText('Facture impayée')).toBeDefined()
		const today = screen.getByRole('region', { name: 'Aujourd’hui' })
		expect(within(today).getByText('Rien ici pour le moment.')).toBeDefined()
	})

	it('filters by module', async () => {
		const user = userEvent.setup()
		await renderWithRouter(
			<ActivityFeedUI activity={ACTIVITY} isLoading={false} />,
		)

		await user.click(screen.getByRole('button', { name: 'Achats' }))

		expect(screen.queryByText('Facture impayée')).toBeNull()
		expect(screen.getByText('Facture fournisseur à valider')).toBeDefined()
	})
})

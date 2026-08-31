import { screen } from '@testing-library/react'
import { Users } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import { AppLauncherUI } from '#/pages/home/ui/app-launcher-ui'
import { renderWithRouter } from '#/test/render-with-router'

describe('AppLauncherUI', () => {
	it('renders one link per item, labelled and routed', async () => {
		await renderWithRouter(
			<AppLauncherUI
				items={[
					{ id: 'crm', label: 'CRM', icon: Users, to: '/o/atelier-bois/crm' },
					{
						id: 'planning',
						label: 'Planning',
						icon: Users,
						to: '/o/atelier-bois/planning',
					},
				]}
			/>,
		)

		const crm = screen.getByRole('link', { name: 'CRM' })
		expect(crm.getAttribute('href')).toBe('/o/atelier-bois/crm')
		expect(screen.getByRole('link', { name: 'Planning' })).toBeDefined()
	})
})

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
					{
						id: 'crm',
						label: 'CRM',
						module: { icon: Users, hue: 255 },
						to: '/o/atelier-bois/crm',
						hint: '2 à suivre',
					},
					{
						id: 'planning',
						label: 'Planning',
						module: { icon: Users },
						to: '/o/atelier-bois/planning',
						hint: 'Rien à suivre',
					},
				]}
			/>,
		)

		const crm = screen.getByRole('link', { name: /^CRM/ })
		expect(crm.getAttribute('href')).toBe('/o/atelier-bois/crm')
		expect(screen.getByText('2 à suivre')).toBeDefined()
		expect(screen.getByRole('link', { name: /^Planning/ })).toBeDefined()
	})
})

import {
	createMemoryHistory,
	createRootRoute,
	createRoute,
	createRouter,
	Outlet,
	RouterProvider,
} from '@tanstack/react-router'
import { render, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'

export async function renderWithRouter(ui: ReactNode, initialPath = '/') {
	const rootRoute = createRootRoute({ component: () => <Outlet /> })
	const component = () => <>{ui}</>
	const routeTree = rootRoute.addChildren([
		createRoute({ getParentRoute: () => rootRoute, path: '/', component }),
		createRoute({ getParentRoute: () => rootRoute, path: '$', component }),
	])
	const router = createRouter({
		routeTree,
		history: createMemoryHistory({ initialEntries: [initialPath] }),
	})

	const result = render(<RouterProvider router={router} />)
	await waitFor(() => {
		if (router.state.status !== 'idle') throw new Error('router not idle')
	})

	return { ...result, router }
}

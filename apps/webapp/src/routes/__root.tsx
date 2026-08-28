import { TanStackDevtools } from '@tanstack/react-devtools'
import type { QueryClient } from '@tanstack/react-query'
import {
	createRootRouteWithContext,
	HeadContent,
	Scripts,
} from '@tanstack/react-router'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { ConfigGate } from '../components/config-gate'
import TanStackQueryDevtools from '../integrations/tanstack-query/devtools'
import appCss from '../styles.css?url'

interface MyRouterContext {
	queryClient: QueryClient
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
	head: () => ({
		meta: [
			{
				charSet: 'utf-8',
			},
			{
				name: 'viewport',
				content: 'width=device-width, initial-scale=1',
			},
			// Chrome's auto-translate mutates the DOM (wraps text nodes in
			// <font>/<span>) independently of React, so a Select/Dialog/
			// Popover opening or closing right after can leave React holding
			// a node reference Chrome already rewrote — React's cleanup then
			// throws removeChild's "not a child of this node". `translate="no"`
			// on <html> stops Chrome from doing that once a translation is
			// active; this meta is the companion piece that stops Chrome
			// from offering the "Translate this page?" prompt in the first
			// place. See https://github.com/radix-ui/primitives/issues/2578
			// and https://github.com/radix-ui/primitives/issues/3795 — the
			// same crash reported directly against Radix's own portals.
			{
				name: 'google',
				content: 'notranslate',
			},
			{
				title: 'Mestier · Console',
			},
			{
				name: 'theme-color',
				content: '#0f3d36',
			},
		],
		links: [
			{
				rel: 'stylesheet',
				href: appCss,
			},
			{
				rel: 'icon',
				type: 'image/svg+xml',
				href: '/icon.svg',
			},
			{
				rel: 'icon',
				type: 'image/x-icon',
				href: '/icon.svg',
			},
			{
				rel: 'apple-touch-icon',
				href: '/icon.svg',
			},
			{
				rel: 'manifest',
				href: '/manifest.json',
			},
		],
	}),
	shellComponent: RootDocument,
})

function RootDocument({ children }: { children: React.ReactNode }) {
	return (
		<html lang="fr" translate="no">
			<head>
				<HeadContent />
			</head>
			<body>
				<ConfigGate>{children}</ConfigGate>
				{import.meta.env.DEV && (
					<TanStackDevtools
						config={{
							position: 'bottom-right',
						}}
						plugins={[
							{
								name: 'Tanstack Router',
								render: <TanStackRouterDevtoolsPanel />,
							},
							TanStackQueryDevtools,
						]}
					/>
				)}
				<Scripts />
			</body>
		</html>
	)
}

import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

window.scrollTo = () => {}

// jsdom n'implémente pas non plus ResizeObserver, dont dépend le Tooltip de
// Radix (via `@radix-ui/react-use-size`) — sans ce stub, un `SidebarMenuButton`
// dont le tooltip s'ouvre (focus au clic, par ex.) plante le rendu et bascule
// sur l'error boundary racine plutôt que de simplement ne rien observer.
if (typeof window.ResizeObserver !== 'function') {
	window.ResizeObserver = class ResizeObserver {
		observe() {}
		unobserve() {}
		disconnect() {}
	}
}

// jsdom n'implémente pas matchMedia, dont dépend `useIsMobile` — donc toute la
// primitive Sidebar. Média toujours faux : les tests rendent la version bureau.
if (typeof window.matchMedia !== 'function') {
	window.matchMedia = (query: string): MediaQueryList =>
		({
			matches: false,
			media: query,
			onchange: null,
			addEventListener: () => {},
			removeEventListener: () => {},
			addListener: () => {},
			removeListener: () => {},
			dispatchEvent: () => false,
		}) as MediaQueryList
}

afterEach(() => {
	cleanup()
})

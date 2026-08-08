import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

window.scrollTo = () => {}

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

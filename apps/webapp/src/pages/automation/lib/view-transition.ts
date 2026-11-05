import { flushSync } from 'react-dom'

export function supportsViewTransition(): boolean {
	return (
		typeof document !== 'undefined' &&
		typeof document.startViewTransition === 'function' &&
		!window.matchMedia('(prefers-reduced-motion: reduce)').matches
	)
}

export function withViewTransition(update: () => void) {
	if (!supportsViewTransition()) {
		update()
		return
	}
	document.startViewTransition(() => flushSync(update))
}

import { type RefObject, useLayoutEffect, useRef } from 'react'

const DURATION_MS = 200
const EASING = 'cubic-bezier(0.2, 0, 0, 1)'
const SELECTOR = '[data-flip-id]'

type Position = { x: number; y: number }

function flipAnimations(element: HTMLElement): Animation[] {
	if (typeof element.getAnimations !== 'function') return []
	return element
		.getAnimations()
		.filter((animation) => !(animation instanceof CSSTransition))
}

export function useFlip(
	containerRef: RefObject<HTMLElement | null>,
	skipIdRef: RefObject<string | null>,
) {
	const previous = useRef(new Map<string, Position>())
	useLayoutEffect(() => {
		const container = containerRef.current
		if (!container) return
		const reduceMotion = window.matchMedia(
			'(prefers-reduced-motion: reduce)',
		).matches
		const elements = [...container.querySelectorAll<HTMLElement>(SELECTOR)]
		const origin = container.getBoundingClientRect()
		const positionOf = (element: HTMLElement): Position => {
			const rect = element.getBoundingClientRect()
			return {
				x: rect.left - origin.left + container.scrollLeft,
				y: rect.top - origin.top + container.scrollTop,
			}
		}
		const inFlight = new Map<string, Position>()
		for (const element of elements) {
			const running = flipAnimations(element)
			if (running.length === 0 || !element.dataset.flipId) continue
			inFlight.set(element.dataset.flipId, positionOf(element))
			for (const animation of running) animation.cancel()
		}
		const next = new Map<string, Position>()
		for (const element of elements) {
			const id = element.dataset.flipId
			if (!id) continue
			const position = positionOf(element)
			next.set(id, position)
			const before = inFlight.get(id) ?? previous.current.get(id)
			if (!before || reduceMotion || id === skipIdRef.current) continue
			const dx = before.x - position.x
			const dy = before.y - position.y
			if (Math.abs(dx) < 1 && Math.abs(dy) < 1) continue
			element.animate(
				[
					{ transform: `translate(${dx}px, ${dy}px)` },
					{ transform: 'translate(0, 0)' },
				],
				{ duration: DURATION_MS, easing: EASING },
			)
		}
		previous.current = next
		skipIdRef.current = null
	})
}

import {
	type PointerEvent as ReactPointerEvent,
	type RefObject,
	useCallback,
	useEffect,
	useRef,
	useState,
} from 'react'
import type { DealStage } from '#/pages/deals/lib/deal-types'

const THRESHOLD_PX = 5
const EDGE_PX = 64
const MAX_SCROLL_PX = 18
const LAND_MS = 180
const EASING = 'cubic-bezier(0.2, 0, 0, 1)'
const COLUMN_SLACK_PX = 6
const BOARD_SLACK_PX = 80
const SWALLOW_CLICK_MS = 60

type Session = {
	id: string
	pointerId: number
	card: HTMLElement
	startX: number
	startY: number
	x: number
	y: number
	offsetX: number
	offsetY: number
	clone: HTMLElement | null
}

function scrollSpeed(distance: number): number {
	return Math.min(
		MAX_SCROLL_PX,
		Math.ceil((distance / EDGE_PX) * MAX_SCROLL_PX),
	)
}

function scrollableParent(element: HTMLElement | null): HTMLElement | null {
	let current = element?.parentElement ?? null
	while (current) {
		const { overflowY } = getComputedStyle(current)
		if (
			(overflowY === 'auto' || overflowY === 'scroll') &&
			current.scrollHeight > current.clientHeight
		)
			return current
		current = current.parentElement
	}
	return null
}

export function useBoardDrag({
	boardRef,
	skipIdRef,
	stageOf,
	onDrop,
}: {
	boardRef: RefObject<HTMLElement | null>
	skipIdRef: RefObject<string | null>
	stageOf: (dealId: string) => DealStage | undefined
	onDrop: (dealId: string, stage: DealStage) => void
}) {
	const [draggedId, setDraggedId] = useState<string | null>(null)
	const [overStage, setOverStage] = useState<DealStage | null>(null)
	const [draggedHeight, setDraggedHeight] = useState(0)
	const session = useRef<Session | null>(null)
	const frame = useRef(0)
	const latest = useRef({ stageOf, onDrop })
	latest.current = { stageOf, onDrop }

	const stageAt = useCallback(
		(x: number, y: number): DealStage | null => {
			const board = boardRef.current
			if (!board) return null
			const bounds = board.getBoundingClientRect()
			if (y < bounds.top - BOARD_SLACK_PX || y > bounds.bottom + BOARD_SLACK_PX)
				return null
			for (const column of board.querySelectorAll<HTMLElement>(
				'[data-stage]',
			)) {
				const rect = column.getBoundingClientRect()
				if (
					x >= rect.left - COLUMN_SLACK_PX &&
					x <= rect.right + COLUMN_SLACK_PX
				)
					return column.dataset.stage as DealStage
			}
			return null
		},
		[boardRef],
	)

	const tick = useCallback(() => {
		const current = session.current
		if (!current?.clone) return
		const board = boardRef.current
		if (board) {
			const bounds = board.getBoundingClientRect()
			if (current.x < bounds.left + EDGE_PX)
				board.scrollLeft -= scrollSpeed(bounds.left + EDGE_PX - current.x)
			else if (current.x > bounds.right - EDGE_PX)
				board.scrollLeft += scrollSpeed(current.x - (bounds.right - EDGE_PX))
			const parent = scrollableParent(board)
			if (parent) {
				const parentBounds = parent.getBoundingClientRect()
				if (current.y < parentBounds.top + EDGE_PX)
					parent.scrollTop -= scrollSpeed(
						parentBounds.top + EDGE_PX - current.y,
					)
				else if (current.y > parentBounds.bottom - EDGE_PX)
					parent.scrollTop += scrollSpeed(
						current.y - (parentBounds.bottom - EDGE_PX),
					)
			}
		}
		current.clone.style.transform = `translate3d(${current.x - current.offsetX}px, ${current.y - current.offsetY}px, 0) rotate(1.5deg)`
		setOverStage(stageAt(current.x, current.y))
		frame.current = requestAnimationFrame(tick)
	}, [boardRef, stageAt])

	const finish = useCallback(
		(dropStage: DealStage | null) => {
			const current = session.current
			session.current = null
			cancelAnimationFrame(frame.current)
			document.body.style.cursor = ''
			document.body.style.userSelect = ''
			if (!current?.clone) return
			const { clone, id } = current
			const board = boardRef.current
			if (dropStage && dropStage !== latest.current.stageOf(id)) {
				skipIdRef.current = id
				latest.current.onDrop(id, dropStage)
			}
			setOverStage(null)
			setDraggedId(null)
			const land = () => {
				const card = board?.querySelector<HTMLElement>(
					`[data-flip-id="${CSS.escape(id)}"]`,
				)
				const release = () => {
					if (card) card.style.visibility = ''
					if (board) board.style.minHeight = ''
					clone.remove()
				}
				if (!card) {
					release()
					return
				}
				card.style.visibility = 'hidden'
				const rect = card.getBoundingClientRect()
				const reduceMotion = window.matchMedia(
					'(prefers-reduced-motion: reduce)',
				).matches
				const animation = clone.animate(
					[
						{ transform: clone.style.transform },
						{
							transform: `translate3d(${rect.left}px, ${rect.top}px, 0) rotate(0deg)`,
						},
					],
					{
						duration: reduceMotion ? 0 : LAND_MS,
						easing: EASING,
						fill: 'forwards',
					},
				)
				animation.onfinish = release
				animation.oncancel = release
			}
			requestAnimationFrame(land)
		},
		[boardRef, skipIdRef],
	)

	const begin = useCallback(
		(current: Session) => {
			const rect = current.card.getBoundingClientRect()
			const clone = current.card.cloneNode(true) as HTMLElement
			clone.removeAttribute('data-flip-id')
			clone.setAttribute('aria-hidden', 'true')
			clone.inert = true
			clone.classList.add('shadow-xl', 'ring-1', 'ring-ring/50')
			Object.assign(clone.style, {
				position: 'fixed',
				left: '0',
				top: '0',
				width: `${rect.width}px`,
				height: `${rect.height}px`,
				margin: '0',
				pointerEvents: 'none',
				zIndex: '60',
				willChange: 'transform',
				transition: 'none',
			})
			clone.style.transform = `translate3d(${rect.left}px, ${rect.top}px, 0)`
			document.body.appendChild(clone)
			current.clone = clone
			current.offsetX = current.startX - rect.left
			current.offsetY = current.startY - rect.top
			const board = boardRef.current
			if (board) board.style.minHeight = `${board.offsetHeight}px`
			document.body.style.cursor = 'grabbing'
			document.body.style.userSelect = 'none'
			setDraggedHeight(rect.height)
			setDraggedId(current.id)
			frame.current = requestAnimationFrame(tick)
		},
		[boardRef, tick],
	)

	const start = useCallback(
		(event: ReactPointerEvent<HTMLElement>, dealId: string) => {
			if (event.button !== 0 || event.pointerType === 'touch') return
			if (!event.currentTarget.contains(event.target as Node)) return
			if ((event.target as HTMLElement).closest('a, [data-no-drag]')) return
			if (session.current) return
			const current: Session = {
				id: dealId,
				pointerId: event.pointerId,
				card: event.currentTarget,
				startX: event.clientX,
				startY: event.clientY,
				x: event.clientX,
				y: event.clientY,
				offsetX: 0,
				offsetY: 0,
				clone: null,
			}
			session.current = current

			const onMove = (move: PointerEvent) => {
				if (move.pointerId !== current.pointerId) return
				current.x = move.clientX
				current.y = move.clientY
				if (
					!current.clone &&
					Math.hypot(current.x - current.startX, current.y - current.startY) >=
						THRESHOLD_PX
				)
					begin(current)
			}
			const detach = () => {
				window.removeEventListener('pointermove', onMove)
				window.removeEventListener('pointerup', onUp)
				window.removeEventListener('pointercancel', onCancel)
				window.removeEventListener('keydown', onKey)
			}
			const onUp = (up: PointerEvent) => {
				if (up.pointerId !== current.pointerId) return
				detach()
				if (!current.clone) {
					session.current = null
					return
				}
				const swallow = (click: Event) => click.stopPropagation()
				window.addEventListener('click', swallow, true)
				setTimeout(
					() => window.removeEventListener('click', swallow, true),
					SWALLOW_CLICK_MS,
				)
				finish(stageAt(up.clientX, up.clientY))
			}
			const onCancel = () => {
				detach()
				finish(null)
			}
			const onKey = (key: KeyboardEvent) => {
				if (key.key === 'Escape') onCancel()
			}
			window.addEventListener('pointermove', onMove)
			window.addEventListener('pointerup', onUp)
			window.addEventListener('pointercancel', onCancel)
			window.addEventListener('keydown', onKey)
		},
		[begin, finish, stageAt],
	)

	useEffect(
		() => () => {
			cancelAnimationFrame(frame.current)
			document.body.style.cursor = ''
			document.body.style.userSelect = ''
			session.current?.clone?.remove()
			session.current = null
		},
		[],
	)

	return { draggedId, overStage, draggedHeight, start }
}

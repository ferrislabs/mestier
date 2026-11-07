import { useRef, useState } from 'react'
import { cn } from '#/lib/utils'
import {
	appliesToPage,
	type Decoration,
	decorationLabel,
	moveRect,
	nudgeRect,
	PAGE_HEIGHT_MM,
	PAGE_WIDTH_MM,
	PAGES_LABELS,
	pagesOf,
	pxToMm,
	type Rect,
	type ResizeHandle,
	rectOf,
	resizeRect,
} from '#/pages/doe-style/lib/decorations'

interface DecorationOverlayProps {
	decorations: Decoration[]
	pageIndex: number
	selectedIndex: number | null
	onSelect: (index: number) => void
	onCommit: (index: number, rect: Rect) => void
}

type Mode = 'move' | ResizeHandle

interface Gesture {
	index: number
	mode: Mode
	startX: number
	startY: number
	widthPx: number
	origin: Rect
	moved: boolean
}

const HANDLES: Array<{ handle: ResizeHandle; className: string }> = [
	{ handle: 'nw', className: '-top-1.5 -left-1.5 cursor-nwse-resize' },
	{
		handle: 'n',
		className: '-top-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize',
	},
	{ handle: 'ne', className: '-top-1.5 -right-1.5 cursor-nesw-resize' },
	{
		handle: 'e',
		className: 'top-1/2 -right-1.5 -translate-y-1/2 cursor-ew-resize',
	},
	{ handle: 'se', className: '-right-1.5 -bottom-1.5 cursor-nwse-resize' },
	{
		handle: 's',
		className: '-bottom-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize',
	},
	{ handle: 'sw', className: '-bottom-1.5 -left-1.5 cursor-nesw-resize' },
	{
		handle: 'w',
		className: 'top-1/2 -left-1.5 -translate-y-1/2 cursor-ew-resize',
	},
]

function frameLabel(decoration: Decoration, index: number): string {
	const scope = PAGES_LABELS[pagesOf(decoration)].toLowerCase()
	return `Élément ${index + 1} : ${decorationLabel(decoration).toLowerCase()}, ${scope}. Flèches pour déplacer d'1 mm, avec Maj de 10 mm.`
}

export function DecorationOverlay({
	decorations,
	pageIndex,
	selectedIndex,
	onSelect,
	onCommit,
}: DecorationOverlayProps) {
	const containerRef = useRef<HTMLDivElement>(null)
	const gesture = useRef<Gesture | null>(null)
	const [draft, setDraft] = useState<{ index: number; rect: Rect } | null>(null)

	function begin(
		event: React.PointerEvent<HTMLElement>,
		index: number,
		mode: Mode,
		decoration: Decoration,
	) {
		if (event.button !== 0) return
		event.stopPropagation()
		onSelect(index)
		const widthPx = containerRef.current?.getBoundingClientRect().width ?? 0
		if (widthPx <= 0) return
		event.currentTarget.setPointerCapture(event.pointerId)
		gesture.current = {
			index,
			mode,
			startX: event.clientX,
			startY: event.clientY,
			widthPx,
			origin: rectOf(decoration),
			moved: false,
		}
	}

	function track(event: React.PointerEvent<HTMLElement>) {
		const current = gesture.current
		if (!current) return
		const dx = pxToMm(event.clientX - current.startX, current.widthPx)
		const dy = pxToMm(event.clientY - current.startY, current.widthPx)
		current.moved = true
		setDraft({
			index: current.index,
			rect:
				current.mode === 'move'
					? moveRect(current.origin, dx, dy)
					: resizeRect(current.origin, current.mode, dx, dy),
		})
	}

	function finish(event: React.PointerEvent<HTMLElement>) {
		const current = gesture.current
		gesture.current = null
		if (event.currentTarget.hasPointerCapture(event.pointerId)) {
			event.currentTarget.releasePointerCapture(event.pointerId)
		}
		const rect = draft
		setDraft(null)
		if (current?.moved && rect && rect.index === current.index) {
			onCommit(rect.index, rect.rect)
		}
	}

	function cancel(event: React.PointerEvent<HTMLElement>) {
		gesture.current = null
		if (event.currentTarget.hasPointerCapture(event.pointerId)) {
			event.currentTarget.releasePointerCapture(event.pointerId)
		}
		setDraft(null)
	}

	function onKeyDown(
		event: React.KeyboardEvent<HTMLElement>,
		index: number,
		decoration: Decoration,
	) {
		if (!event.key.startsWith('Arrow')) return
		event.preventDefault()
		const rect = rectOf(decoration)
		const next = nudgeRect(rect, event.key, event.shiftKey)
		if (next.x !== rect.x || next.y !== rect.y) onCommit(index, next)
	}

	return (
		<div ref={containerRef} className="absolute inset-0">
			{decorations.map((decoration, index) => {
				if (!appliesToPage(decoration, pageIndex)) return null
				const selected = index === selectedIndex
				const rect =
					draft && draft.index === index ? draft.rect : rectOf(decoration)
				return (
					<button
						// biome-ignore lint/suspicious/noArrayIndexKey: stacking order is positional
						key={index}
						type="button"
						aria-label={frameLabel(decoration, index)}
						aria-pressed={selected}
						style={{
							left: `${(rect.x / PAGE_WIDTH_MM) * 100}%`,
							top: `${(rect.y / PAGE_HEIGHT_MM) * 100}%`,
							width: `${(rect.width / PAGE_WIDTH_MM) * 100}%`,
							height: `${(rect.height / PAGE_HEIGHT_MM) * 100}%`,
						}}
						className={cn(
							'absolute cursor-move touch-none rounded-[1px] outline-none focus-visible:ring-2 focus-visible:ring-ring',
							selected
								? 'z-10 outline-2 outline-primary outline-dashed'
								: 'outline-1 outline-transparent hover:outline-primary/50 hover:outline-dashed',
						)}
						onPointerDown={(event) => begin(event, index, 'move', decoration)}
						onPointerMove={track}
						onPointerUp={finish}
						onPointerCancel={cancel}
						onFocus={() => onSelect(index)}
						onKeyDown={(event) => onKeyDown(event, index, decoration)}
					>
						{selected
							? HANDLES.map(({ handle, className }) => (
									<span
										key={handle}
										aria-hidden
										data-handle={handle}
										className={cn(
											'absolute size-3 rounded-sm border border-primary bg-background',
											className,
										)}
										onPointerDown={(event) =>
											begin(event, index, handle, decoration)
										}
										onPointerMove={track}
										onPointerUp={finish}
										onPointerCancel={cancel}
									/>
								))
							: null}
					</button>
				)
			})}
		</div>
	)
}

import { Fragment, useRef, useState } from 'react'
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
import {
	collectGuides,
	type GuideLine,
	type GuideSet,
	snapRect,
} from '#/pages/doe-style/lib/snap'
import {
	angleFromPointer,
	cssTransform,
	rotatedBounds,
	rotationOf,
	snapAngle,
	toLocalDelta,
} from '#/pages/doe-style/lib/transform'

interface DecorationOverlayProps {
	decorations: Decoration[]
	pageIndex: number
	selectedIndex: number | null
	onSelect: (index: number) => void
	onCommit: (index: number, rect: Rect) => void
	onRotate?: (index: number, rotation: number) => void
	snapping?: boolean
}

type Mode = 'move' | 'rotate' | ResizeHandle

interface Draft {
	index: number
	rect: Rect
	rotation: number
	guides: GuideLine[]
}

interface Gesture {
	index: number
	mode: Mode
	startX: number
	startY: number
	widthPx: number
	origin: Rect
	rotation: number
	left: number
	top: number
	guides: GuideSet | null
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
	onRotate,
	snapping = false,
}: DecorationOverlayProps) {
	const containerRef = useRef<HTMLDivElement>(null)
	const gesture = useRef<Gesture | null>(null)
	const [draft, setDraft] = useState<Draft | null>(null)

	function begin(
		event: React.PointerEvent<HTMLElement>,
		index: number,
		mode: Mode,
		decoration: Decoration,
	) {
		if (event.button !== 0) return
		event.stopPropagation()
		onSelect(index)
		const bounds = containerRef.current?.getBoundingClientRect()
		const widthPx = bounds?.width ?? 0
		if (!bounds || widthPx <= 0) return
		event.currentTarget.setPointerCapture(event.pointerId)
		gesture.current = {
			index,
			mode,
			startX: event.clientX,
			startY: event.clientY,
			widthPx,
			origin: rectOf(decoration),
			rotation: rotationOf(decoration),
			left: bounds.left,
			top: bounds.top,
			guides: snapping ? collectGuides(decorations, pageIndex, index) : null,
			moved: false,
		}
	}

	function track(event: React.PointerEvent<HTMLElement>) {
		const current = gesture.current
		if (!current) return
		current.moved = true
		if (current.mode === 'rotate') {
			const scale = current.widthPx / PAGE_WIDTH_MM
			const center = {
				x: current.left + (current.origin.x + current.origin.width / 2) * scale,
				y: current.top + (current.origin.y + current.origin.height / 2) * scale,
			}
			const angle = angleFromPointer(center, {
				x: event.clientX,
				y: event.clientY,
			})
			setDraft({
				index: current.index,
				rect: current.origin,
				rotation: snapAngle(angle, event.shiftKey),
				guides: [],
			})
			return
		}
		const dx = pxToMm(event.clientX - current.startX, current.widthPx)
		const dy = pxToMm(event.clientY - current.startY, current.widthPx)
		if (current.mode !== 'move') {
			const local = toLocalDelta(dx, dy, current.rotation)
			setDraft({
				index: current.index,
				rect: resizeRect(current.origin, current.mode, local.dx, local.dy),
				rotation: current.rotation,
				guides: [],
			})
			return
		}
		let rect = moveRect(current.origin, dx, dy)
		let guides: GuideLine[] = []
		if (current.guides && !event.altKey) {
			const snap = snapRect(
				rotatedBounds(rect, current.rotation),
				current.guides,
			)
			rect = moveRect(rect, snap.dx, snap.dy)
			guides = snap.lines
		}
		setDraft({ index: current.index, rect, rotation: current.rotation, guides })
	}

	function finish(event: React.PointerEvent<HTMLElement>) {
		const current = gesture.current
		gesture.current = null
		if (event.currentTarget.hasPointerCapture(event.pointerId)) {
			event.currentTarget.releasePointerCapture(event.pointerId)
		}
		const result = draft
		setDraft(null)
		if (!current?.moved || !result || result.index !== current.index) return
		if (current.mode === 'rotate') onRotate?.(result.index, result.rotation)
		else onCommit(result.index, result.rect)
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
				const active = draft && draft.index === index ? draft : null
				const rect = active ? active.rect : rectOf(decoration)
				const rotation = active ? active.rotation : rotationOf(decoration)
				const turned = { ...decoration, rotation }
				const frame = {
					left: `${(rect.x / PAGE_WIDTH_MM) * 100}%`,
					top: `${(rect.y / PAGE_HEIGHT_MM) * 100}%`,
					width: `${(rect.width / PAGE_WIDTH_MM) * 100}%`,
					height: `${(rect.height / PAGE_HEIGHT_MM) * 100}%`,
				}
				return (
					<Fragment
						// biome-ignore lint/suspicious/noArrayIndexKey: stacking order is positional
						key={index}
					>
						<button
							type="button"
							aria-label={frameLabel(decoration, index)}
							aria-pressed={selected}
							style={{ ...frame, transform: cssTransform(turned) }}
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
						/>
						{selected ? (
							<div
								style={{ ...frame, transform: `rotate(${rotation}deg)` }}
								className="pointer-events-none absolute z-20"
							>
								{HANDLES.map(({ handle, className }) => (
									<span
										key={handle}
										aria-hidden
										data-handle={handle}
										className={cn(
											'pointer-events-auto absolute size-3 touch-none rounded-sm border border-primary bg-background',
											className,
										)}
										onPointerDown={(event) =>
											begin(event, index, handle, decoration)
										}
										onPointerMove={track}
										onPointerUp={finish}
										onPointerCancel={cancel}
									/>
								))}
								{onRotate ? (
									<>
										<span
											aria-hidden
											className="absolute -top-6 left-1/2 h-6 w-px -translate-x-1/2 bg-primary"
										/>
										<span
											aria-hidden
											data-handle="rotate"
											title="Pivoter (Maj : angle libre)"
											className="pointer-events-auto absolute -top-9 left-1/2 size-4 -translate-x-1/2 cursor-grab touch-none rounded-full border-2 border-primary bg-background active:cursor-grabbing"
											onPointerDown={(event) =>
												begin(event, index, 'rotate', decoration)
											}
											onPointerMove={track}
											onPointerUp={finish}
											onPointerCancel={cancel}
										/>
									</>
								) : null}
							</div>
						) : null}
					</Fragment>
				)
			})}
			{draft?.guides.map((line) => (
				<span
					key={`${line.axis}-${line.position}`}
					aria-hidden
					data-guide={line.axis}
					style={
						line.axis === 'x'
							? { left: `${(line.position / PAGE_WIDTH_MM) * 100}%` }
							: { top: `${(line.position / PAGE_HEIGHT_MM) * 100}%` }
					}
					className={cn(
						'pointer-events-none absolute z-30 bg-fuchsia-500',
						line.axis === 'x' ? 'inset-y-0 w-px' : 'inset-x-0 h-px',
					)}
				/>
			))}
		</div>
	)
}

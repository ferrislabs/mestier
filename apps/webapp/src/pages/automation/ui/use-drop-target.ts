import { type DragEvent, useState } from 'react'

export function useDropTarget({
	accepts,
	onDrop,
}: {
	accepts: boolean
	onDrop: () => void
}) {
	const [over, setOver] = useState(false)

	return {
		over: accepts && over,
		handlers: {
			onDragOver: (event: DragEvent) => {
				if (!accepts) return
				event.preventDefault()
				event.dataTransfer.dropEffect = 'move'
				setOver(true)
			},
			onDragLeave: () => setOver(false),
			onDrop: (event: DragEvent) => {
				setOver(false)
				if (!accepts) return
				event.preventDefault()
				onDrop()
			},
		},
	}
}

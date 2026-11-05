import type { ReactNode } from 'react'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'

export interface MenuAnchor {
	x: number
	y: number
}

export function ContextMenuAt({
	anchor,
	label,
	onClose,
	children,
}: {
	anchor: MenuAnchor | null
	label: string
	onClose: () => void
	children: ReactNode
}) {
	return (
		<DropdownMenu
			open={anchor !== null}
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DropdownMenuTrigger asChild>
				<span
					aria-hidden
					className="pointer-events-none absolute size-px"
					style={{ left: anchor?.x ?? 0, top: anchor?.y ?? 0 }}
				/>
			</DropdownMenuTrigger>
			<DropdownMenuContent
				align="start"
				sideOffset={2}
				aria-label={label}
				className="w-52"
			>
				{children}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

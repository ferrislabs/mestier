import { MoreHorizontal } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'

export interface RowAction {
	label: string
	icon?: ReactNode
	destructive?: boolean
	onSelect: () => void
}

export function RowActions({
	label,
	actions,
}: {
	label: string
	actions: readonly RowAction[]
}) {
	if (actions.length === 0) return null
	return (
		<div className="flex justify-end opacity-0 transition focus-within:opacity-100 group-focus-within:opacity-100 group-hover:opacity-100">
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<Button
						size="icon-sm"
						variant="ghost"
						onClick={(event) => event.stopPropagation()}
					>
						<MoreHorizontal />
						<span className="sr-only">Actions {label}</span>
					</Button>
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					{actions.map((action) => (
						<DropdownMenuItem
							key={action.label}
							variant={action.destructive ? 'destructive' : 'default'}
							onClick={(event) => {
								event.stopPropagation()
								action.onSelect()
							}}
						>
							{action.icon}
							{action.label}
						</DropdownMenuItem>
					))}
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	)
}

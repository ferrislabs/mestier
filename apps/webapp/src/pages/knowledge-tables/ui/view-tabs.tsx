import {
	Ellipsis,
	KanbanSquare,
	Pencil,
	Plus,
	Sheet,
	Trash2,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { cn } from '#/lib/utils'
import type { TableView } from '#/pages/knowledge-tables/lib/types'

interface ViewTabsProps {
	views: readonly TableView[]
	activeId: string | null
	canManage: boolean
	onSelect: (viewId: string) => void
	onCreate: () => void
	onRename: (view: TableView) => void
	onDelete: (view: TableView) => void
}

export function ViewTabs({
	views,
	activeId,
	canManage,
	onSelect,
	onCreate,
	onRename,
	onDelete,
}: ViewTabsProps) {
	const isLast = views.length <= 1
	return (
		<div className="flex flex-wrap items-center gap-1 border-b">
			<div role="tablist" aria-label="Vues" className="flex flex-wrap gap-1">
				{views.map((view) => {
					const isActive = view.id === activeId
					const Icon = view.kind === 'kanban' ? KanbanSquare : Sheet
					return (
						<div
							key={view.id}
							className={cn(
								'-mb-px flex items-center border-b-2',
								isActive ? 'border-primary' : 'border-transparent',
							)}
						>
							<button
								type="button"
								role="tab"
								aria-selected={isActive}
								className={cn(
									'flex items-center gap-1.5 rounded-t-md px-2.5 py-1.5 text-sm outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring',
									isActive ? 'font-medium' : 'text-muted-foreground',
								)}
								onClick={() => onSelect(view.id)}
							>
								<Icon aria-hidden className="size-4" />
								{view.name}
							</button>
							{isActive && canManage ? (
								<DropdownMenu>
									<DropdownMenuTrigger asChild>
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											aria-label={`Actions de la vue ${view.name}`}
										>
											<Ellipsis aria-hidden />
										</Button>
									</DropdownMenuTrigger>
									<DropdownMenuContent align="start">
										<DropdownMenuItem onSelect={() => onRename(view)}>
											<Pencil aria-hidden /> Renommer
										</DropdownMenuItem>
										<DropdownMenuSeparator />
										<DropdownMenuItem
											variant="destructive"
											disabled={isLast}
											title={
												isLast ? 'Une table garde au moins une vue.' : undefined
											}
											onSelect={() => onDelete(view)}
										>
											<Trash2 aria-hidden /> Supprimer
										</DropdownMenuItem>
									</DropdownMenuContent>
								</DropdownMenu>
							) : null}
						</div>
					)
				})}
			</div>
			{canManage ? (
				<Button type="button" variant="ghost" size="sm" onClick={onCreate}>
					<Plus aria-hidden />
					Nouvelle vue
				</Button>
			) : null}
		</div>
	)
}

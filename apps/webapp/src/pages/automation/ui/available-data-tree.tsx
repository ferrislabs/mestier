import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import type { DataTreeNode } from '#/pages/automation/lib/data-tree'

export interface AvailableDataTreeProps {
	branches: DataTreeNode[]
	onInsert: (path: string) => void
}

export function AvailableDataTree({
	branches,
	onInsert,
}: AvailableDataTreeProps) {
	return (
		<div className="flex flex-col gap-2">
			{branches.length === 0 ? (
				<p className="text-sm text-muted-foreground">
					Aucune donnée disponible.
				</p>
			) : (
				<ul className="flex flex-col gap-0.5">
					{branches.map((node) => (
						<TreeNodeItem key={node.path} node={node} onInsert={onInsert} />
					))}
				</ul>
			)}
		</div>
	)
}

function TreeNodeItem({
	node,
	onInsert,
}: {
	node: DataTreeNode
	onInsert: (path: string) => void
}) {
	const [open, setOpen] = useState(false)

	if (node.kind === 'notice') {
		return (
			<li className="flex flex-col gap-0.5 px-1.5 py-1 text-xs text-muted-foreground">
				<span className="font-medium text-foreground">{node.label}</span>
				<span>{node.message}</span>
			</li>
		)
	}

	if (node.kind === 'leaf') {
		return (
			<li>
				<button
					type="button"
					draggable
					aria-label={`Insérer ${node.path}`}
					onDragStart={(event) => {
						event.dataTransfer.setData('text/plain', node.path)
					}}
					onClick={() => onInsert(node.path)}
					className="flex w-full items-center gap-1.5 rounded px-1.5 py-1 text-left text-xs text-foreground hover:bg-accent"
				>
					{node.label}
				</button>
			</li>
		)
	}

	return (
		<li>
			<button
				type="button"
				aria-expanded={open}
				onClick={() => setOpen((current) => !current)}
				className="flex w-full items-center gap-1 rounded px-1.5 py-1 text-left text-xs font-medium hover:bg-accent"
			>
				{open ? (
					<ChevronDown className="size-3.5 shrink-0" />
				) : (
					<ChevronRight className="size-3.5 shrink-0" />
				)}
				{node.label}
			</button>
			{open ? (
				<ul className="flex flex-col gap-0.5 border-l pl-3">
					{node.children.map((child) => (
						<TreeNodeItem key={child.path} node={child} onInsert={onInsert} />
					))}
				</ul>
			) : null}
		</li>
	)
}

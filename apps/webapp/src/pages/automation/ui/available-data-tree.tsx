import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import { cn } from '#/lib/utils'
import type { DataTreeNode } from '#/pages/automation/lib/data-tree'

export type DataTreeSource = 'example' | 'last_run'

export interface AvailableDataTreeProps {
	branches: DataTreeNode[]
	source: DataTreeSource
	hasLastRun: boolean
	onSourceChange: (source: DataTreeSource) => void
	onInsert: (path: string) => void
}

export function AvailableDataTree({
	branches,
	source,
	hasLastRun,
	onSourceChange,
	onInsert,
}: AvailableDataTreeProps) {
	return (
		<div className="flex flex-col gap-2">
			<div className="flex gap-1 rounded-md border p-0.5 text-xs">
				<SourceButton
					label="Exemple"
					active={source === 'example'}
					onClick={() => onSourceChange('example')}
				/>
				<SourceButton
					label="Dernière exécution"
					active={source === 'last_run'}
					disabled={!hasLastRun}
					onClick={() => onSourceChange('last_run')}
				/>
			</div>
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

function SourceButton({
	label,
	active,
	disabled = false,
	onClick,
}: {
	label: string
	active: boolean
	disabled?: boolean
	onClick: () => void
}) {
	return (
		<button
			type="button"
			aria-pressed={active}
			disabled={disabled}
			onClick={onClick}
			className={cn(
				'flex-1 rounded px-2 py-1',
				active ? 'bg-accent font-medium' : 'text-muted-foreground',
				'disabled:cursor-not-allowed disabled:opacity-50',
			)}
		>
			{label}
		</button>
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

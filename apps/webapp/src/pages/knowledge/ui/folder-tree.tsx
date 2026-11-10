import { ChevronRight, Folder, FolderOpen, HardDrive } from 'lucide-react'
import { type ReactNode, useState } from 'react'
import type { KnowledgeFolder } from '#/hooks/use-knowledge'
import { cn } from '#/lib/utils'
import {
	carriesArticles,
	childrenByParent,
	readDraggedArticles,
} from '#/pages/knowledge/lib/folders'

interface FolderTreeProps {
	folders: readonly KnowledgeFolder[]
	/** `null` is the root. */
	selectedId: string | null
	expanded: ReadonlySet<string>
	onSelect: (id: string | null) => void
	onToggle: (id: string) => void
	/** Absent when articles cannot be moved. */
	onDropArticles?: (folderId: string | null, articleIds: string[]) => void
}

/** A row that accepts a drop of articles and lights up while hovered by one. */
export function DropRow({
	folderId,
	onDropArticles,
	className,
	children,
}: {
	folderId: string | null
	onDropArticles?: (folderId: string | null, articleIds: string[]) => void
	className?: string
	children: ReactNode
}) {
	const [isOver, setIsOver] = useState(false)
	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: HTML5 drag & drop's drop target has no native interactive equivalent; the keyboard path is the "Déplacer vers" menu.
		<div
			className={cn(
				'flex items-center rounded-md',
				isOver && 'bg-primary/10 ring-1 ring-primary',
				className,
			)}
			onDragOver={(event) => {
				if (!onDropArticles || !carriesArticles(event.dataTransfer)) return
				event.preventDefault()
				event.dataTransfer.dropEffect = 'move'
				setIsOver(true)
			}}
			onDragLeave={() => setIsOver(false)}
			onDrop={(event) => {
				setIsOver(false)
				if (!onDropArticles) return
				const ids = readDraggedArticles(event.dataTransfer)
				if (ids.length === 0) return
				event.preventDefault()
				onDropArticles(folderId, ids)
			}}
		>
			{children}
		</div>
	)
}

export function FolderTree({
	folders,
	selectedId,
	expanded,
	onSelect,
	onToggle,
	onDropArticles,
}: FolderTreeProps) {
	const byParent = childrenByParent(folders)

	const branch = (parentId: string | null, depth: number) => {
		const children = byParent.get(parentId) ?? []
		if (children.length === 0) return null
		return (
			<ul className="grid gap-0.5">
				{children.map((folder) => {
					const hasChildren = (byParent.get(folder.id)?.length ?? 0) > 0
					const isOpen = expanded.has(folder.id)
					const isSelected = folder.id === selectedId
					const Icon = isSelected ? FolderOpen : Folder
					return (
						<li key={folder.id}>
							<DropRow folderId={folder.id} onDropArticles={onDropArticles}>
								<span style={{ width: depth * 12 }} className="shrink-0" />
								{hasChildren ? (
									<button
										type="button"
										aria-label={
											isOpen
												? `Replier ${folder.name}`
												: `Déplier ${folder.name}`
										}
										aria-expanded={isOpen}
										className="grid size-6 shrink-0 place-items-center rounded text-muted-foreground hover:bg-muted"
										onClick={() => onToggle(folder.id)}
									>
										<ChevronRight
											aria-hidden
											className={cn(
												'size-3.5 transition',
												isOpen && 'rotate-90',
											)}
										/>
									</button>
								) : (
									<span className="size-6 shrink-0" />
								)}
								<button
									type="button"
									className={cn(
										'flex min-w-0 flex-1 items-center gap-2 rounded px-1.5 py-1 text-left text-sm hover:bg-muted',
										isSelected && 'bg-muted font-medium',
									)}
									aria-current={isSelected ? 'page' : undefined}
									onClick={() => onSelect(folder.id)}
								>
									<Icon
										aria-hidden
										className="size-4 shrink-0 text-muted-foreground"
									/>
									<span className="truncate">{folder.name}</span>
								</button>
							</DropRow>
							{hasChildren && isOpen ? branch(folder.id, depth + 1) : null}
						</li>
					)
				})}
			</ul>
		)
	}

	return (
		<nav aria-label="Dossiers" className="grid gap-0.5">
			<DropRow folderId={null} onDropArticles={onDropArticles}>
				<button
					type="button"
					className={cn(
						'flex flex-1 items-center gap-2 rounded px-1.5 py-1 text-left text-sm hover:bg-muted',
						selectedId === null && 'bg-muted font-medium',
					)}
					aria-current={selectedId === null ? 'page' : undefined}
					onClick={() => onSelect(null)}
				>
					<HardDrive aria-hidden className="size-4 text-muted-foreground" />
					Drive
				</button>
			</DropRow>
			{branch(null, 0)}
		</nav>
	)
}

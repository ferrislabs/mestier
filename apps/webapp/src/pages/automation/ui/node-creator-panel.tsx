import { ArrowLeft, ArrowRight, Library, Search, X } from 'lucide-react'
import { type ReactNode, useEffect, useRef } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import type { CollectionOperation } from '#/pages/automation/lib/collections'
import type {
	CreatorCategory,
	CreatorKind,
} from '#/pages/automation/lib/creator'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import { MethodBadge } from '#/pages/automation/ui/method-badge'

export type NodeCreatorView =
	| {
			type: 'categories'
			categories: readonly CreatorCategory[]
			triggerEntry: CreatorCategory
	  }
	| { type: 'kinds'; kinds: readonly CreatorKind[] }
	| { type: 'collections'; collections: readonly CollectionEntry[] }
	| { type: 'operations'; operations: readonly OperationEntry[] }
	| {
			type: 'results'
			groups: readonly CreatorCategory[]
			operations: readonly OperationEntry[]
	  }

export type CollectionEntry = {
	id: string
	title: string
	operationCount: number
}

export type OperationEntry = {
	collectionId: string
	collectionTitle: string
	operation: CollectionOperation
}

const ROW =
	'flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring'

function CategoryRow({
	category,
	onPick,
}: {
	category: CreatorCategory
	onPick: () => void
}) {
	const Icon = category.icon
	return (
		<button type="button" onClick={onPick} className={ROW}>
			<Icon
				className="size-5 shrink-0 text-muted-foreground"
				strokeWidth={1.6}
				aria-hidden
			/>
			<span className="min-w-0 flex-1">
				<span className="block font-medium text-sm">{category.label}</span>
				<span className="block text-muted-foreground text-xs leading-snug">
					{category.description}
				</span>
			</span>
			<ArrowRight
				className="size-4 shrink-0 text-muted-foreground"
				aria-hidden
			/>
		</button>
	)
}

function KindRow({ kind, onPick }: { kind: CreatorKind; onPick: () => void }) {
	return (
		<button type="button" onClick={onPick} className={ROW}>
			<KindIconTile icon={kind.icon} hue={kind.hue} size="sm" />
			<span className="min-w-0 flex-1">
				<span className="block truncate font-medium text-sm">{kind.label}</span>
				<span className="block text-muted-foreground text-xs leading-snug">
					{kind.description}
				</span>
			</span>
		</button>
	)
}

export function NodeCreatorPanel({
	title,
	description,
	onBack,
	query,
	onQueryChange,
	view,
	onPickCategory,
	onPickKind,
	onPickCollection,
	onPickOperation,
	emptyCollectionsAction,
	onClose,
}: {
	title: string
	description?: string
	onBack?: () => void
	query: string
	onQueryChange: (query: string) => void
	view: NodeCreatorView
	onPickCategory: (category: CreatorCategory) => void
	onPickKind: (kind: CreatorKind) => void
	onPickCollection: (collectionId: string) => void
	onPickOperation: (entry: OperationEntry) => void
	emptyCollectionsAction: ReactNode
	onClose: () => void
}) {
	const searchRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		searchRef.current?.focus()
	}, [])

	const renderOperation = (entry: OperationEntry, showCollection: boolean) => (
		<li key={`${entry.collectionId}-${entry.operation.id}`}>
			<button
				type="button"
				onClick={() => onPickOperation(entry)}
				className={ROW}
			>
				<MethodBadge method={entry.operation.method} />
				<span className="min-w-0 flex-1">
					<span className="block truncate font-medium text-sm">
						{entry.operation.summary}
					</span>
					<span className="block truncate font-mono text-muted-foreground text-xs">
						{showCollection ? `${entry.collectionTitle} · ` : ''}
						{entry.operation.path}
					</span>
				</span>
			</button>
		</li>
	)

	const renderKind = (kind: CreatorKind) => (
		<li key={kind.id}>
			<KindRow kind={kind} onPick={() => onPickKind(kind)} />
		</li>
	)

	return (
		<aside
			aria-label="Créateur de nœuds"
			onKeyDown={(event) => {
				if (event.key === 'Escape') onClose()
			}}
			className="fade-in slide-in-from-right-4 absolute inset-y-0 right-0 z-20 flex w-[360px] max-w-full animate-in flex-col border-border border-l bg-card shadow-lg duration-200"
		>
			<header className="flex items-start gap-2 border-border border-b bg-muted/50 px-3 py-3">
				{onBack && (
					<Button
						variant="ghost"
						size="icon-sm"
						onClick={onBack}
						aria-label="Revenir aux catégories"
					>
						<ArrowLeft />
					</Button>
				)}
				<div className="min-w-0 flex-1 py-1 pl-1">
					<h2 className="font-semibold text-base leading-tight">{title}</h2>
					{description && (
						<p className="mt-1 text-muted-foreground text-xs leading-snug">
							{description}
						</p>
					)}
				</div>
				<Button
					variant="ghost"
					size="icon-sm"
					onClick={onClose}
					aria-label="Fermer le panneau des nœuds"
				>
					<X />
				</Button>
			</header>
			<div className="relative px-3 py-3">
				<Search
					className="pointer-events-none absolute top-1/2 left-[22px] size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					ref={searchRef}
					type="search"
					value={query}
					onChange={(event) => onQueryChange(event.target.value)}
					placeholder="Rechercher un nœud…"
					aria-label="Rechercher un nœud"
					className="pl-8"
				/>
			</div>
			<div className="min-h-0 flex-1 overflow-y-auto px-2 pb-4">
				{view.type === 'categories' && (
					<>
						<ul>
							{view.categories.map((category) => (
								<li key={category.id}>
									<CategoryRow
										category={category}
										onPick={() => onPickCategory(category)}
									/>
								</li>
							))}
						</ul>
						<div className="mx-3 my-2 border-border border-t" />
						<CategoryRow
							category={{
								...view.triggerEntry,
								label: 'Ajouter un déclencheur',
							}}
							onPick={() => onPickCategory(view.triggerEntry)}
						/>
					</>
				)}
				{view.type === 'kinds' && <ul>{view.kinds.map(renderKind)}</ul>}
				{view.type === 'collections' &&
					(view.collections.length === 0 ? (
						<div className="grid justify-items-center gap-3 px-3 py-8 text-center text-muted-foreground text-sm">
							<p>Aucune collection importée pour le moment.</p>
							{emptyCollectionsAction}
						</div>
					) : (
						<ul>
							{view.collections.map((collection) => (
								<li key={collection.id}>
									<button
										type="button"
										onClick={() => onPickCollection(collection.id)}
										className={ROW}
									>
										<Library
											className="size-5 shrink-0 text-muted-foreground"
											strokeWidth={1.6}
											aria-hidden
										/>
										<span className="min-w-0 flex-1">
											<span className="block truncate font-medium text-sm">
												{collection.title}
											</span>
											<span className="block text-muted-foreground text-xs">
												{collection.operationCount} opération
												{collection.operationCount > 1 ? 's' : ''}
											</span>
										</span>
										<ArrowRight
											className="size-4 shrink-0 text-muted-foreground"
											aria-hidden
										/>
									</button>
								</li>
							))}
						</ul>
					))}
				{view.type === 'operations' && (
					<ul>
						{view.operations.map((entry) => renderOperation(entry, false))}
					</ul>
				)}
				{view.type === 'results' &&
					(view.groups.length === 0 && view.operations.length === 0 ? (
						<p className="px-2 py-8 text-center text-muted-foreground text-sm">
							Aucun nœud ne correspond à « {query.trim()} ».
						</p>
					) : (
						<>
							{view.groups.map((group) => (
								<section
									key={group.id}
									aria-label={group.label}
									className="mb-2"
								>
									<h3 className="px-3 pt-2 pb-1 font-medium text-muted-foreground text-xs">
										{group.label}
									</h3>
									<ul>{group.kinds.map(renderKind)}</ul>
								</section>
							))}
							{view.operations.length > 0 && (
								<section
									aria-label="Opérations de collections"
									className="mb-2"
								>
									<h3 className="px-3 pt-2 pb-1 font-medium text-muted-foreground text-xs">
										Opérations de collections
									</h3>
									<ul>
										{view.operations.map((entry) =>
											renderOperation(entry, true),
										)}
									</ul>
								</section>
							)}
						</>
					))}
			</div>
		</aside>
	)
}

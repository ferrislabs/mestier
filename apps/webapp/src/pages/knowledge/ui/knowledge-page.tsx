import {
	ChevronLeft,
	ChevronRight,
	Ellipsis,
	FileText,
	Folder,
	FolderInput,
	FolderPlus,
	Loader2,
	NotebookText,
	Pencil,
	Plus,
	Search,
	Trash2,
	X,
} from 'lucide-react'
import { useId } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { KnowledgeArticle, KnowledgeFolder } from '#/hooks/use-knowledge'
import { cn } from '#/lib/utils'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import { ACCEPT, fileTypeLabel, formatSize } from '#/pages/knowledge/lib/files'
import { ARTICLE_DRAG_TYPE } from '#/pages/knowledge/lib/folders'
import { DropRow, FolderTree } from '#/pages/knowledge/ui/folder-tree'
import type { UploadItem } from '#/pages/photo-library/lib/uploads'
import { UploadList } from '#/pages/photo-library/ui/upload-list'
import { Page, PageHeader } from '#/pages/projects/ui/page'

interface KnowledgePageProps {
	folders: readonly KnowledgeFolder[]
	/** `null` is the root of the drive. */
	currentFolderId: string | null
	/** From the root down to the open folder. Empty at the root. */
	path: readonly KnowledgeFolder[]
	/** The folders directly inside the open one. */
	subfolders: readonly KnowledgeFolder[]
	expanded: ReadonlySet<string>
	articles: readonly KnowledgeArticle[]
	selected: ReadonlySet<string>
	canManage: boolean
	canCreateFolder: boolean
	uploads: readonly UploadItem[]
	isUploading: boolean
	isLoading: boolean
	error: string | null
	hasFilters: boolean
	page: number
	hasNext: boolean
	total: number | null
	q: string
	tagInput: string
	onOpenFolder: (folderId: string | null) => void
	onToggleFolder: (folderId: string) => void
	onNewFolder: () => void
	onRenameFolder: (folder: KnowledgeFolder) => void
	onMoveFolder: (folder: KnowledgeFolder) => void
	onDeleteFolder: (folder: KnowledgeFolder) => void
	onDropArticles: (folderId: string | null, articleIds: string[]) => void
	onToggleSelected: (articleId: string) => void
	onClearSelection: () => void
	onMoveSelected: () => void
	onFiles: (files: File[]) => void
	onClearUploads: () => void
	onNewDocument: () => void
	isCreatingDocument: boolean
	onQChange: (q: string) => void
	onTagInputChange: (tagInput: string) => void
	onPageChange: (page: number) => void
	onOpen: (article: KnowledgeArticle) => void
}

function formatDate(iso: string): string {
	return new Date(iso).toLocaleDateString('fr-FR')
}

function FolderRow({
	folder,
	canManage,
	onOpen,
	onRename,
	onMove,
	onDelete,
	onDropArticles,
}: {
	folder: KnowledgeFolder
	canManage: boolean
	onOpen: () => void
	onRename: () => void
	onMove: () => void
	onDelete: () => void
	onDropArticles?: (folderId: string | null, articleIds: string[]) => void
}) {
	return (
		<li>
			<DropRow
				folderId={folder.id}
				onDropArticles={onDropArticles}
				className="gap-2 px-4 py-2"
			>
				<button
					type="button"
					onClick={onOpen}
					className="flex min-w-0 flex-1 items-center gap-3 rounded py-1 text-left outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
				>
					<Folder
						aria-hidden
						className="size-4 shrink-0 text-muted-foreground"
					/>
					<span className="truncate font-medium">{folder.name}</span>
				</button>
				{canManage ? (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label={`Actions du dossier ${folder.name}`}
							>
								<Ellipsis aria-hidden />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuItem onSelect={onRename}>
								<Pencil aria-hidden /> Renommer
							</DropdownMenuItem>
							<DropdownMenuItem onSelect={onMove}>
								<FolderInput aria-hidden /> Déplacer
							</DropdownMenuItem>
							<DropdownMenuSeparator />
							<DropdownMenuItem variant="destructive" onSelect={onDelete}>
								<Trash2 aria-hidden /> Supprimer
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				) : null}
			</DropRow>
		</li>
	)
}

function ArticleRow({
	article,
	canManage,
	isSelected,
	selected,
	onToggle,
	onOpen,
}: {
	article: KnowledgeArticle
	canManage: boolean
	isSelected: boolean
	selected: ReadonlySet<string>
	onToggle: () => void
	onOpen: (article: KnowledgeArticle) => void
}) {
	const Icon = article.kind === 'document' ? NotebookText : FileText
	return (
		<li
			draggable={canManage}
			onDragStart={(event) => {
				const ids = isSelected ? [...selected] : [article.id]
				event.dataTransfer.setData(ARTICLE_DRAG_TYPE, JSON.stringify(ids))
				event.dataTransfer.effectAllowed = 'move'
			}}
			className={cn(
				'flex items-start gap-3 px-4',
				isSelected && 'bg-primary/5',
			)}
		>
			{canManage ? (
				<Checkbox
					checked={isSelected}
					onCheckedChange={onToggle}
					aria-label={`Sélectionner ${article.title}`}
					className="mt-3.5"
				/>
			) : null}
			<button
				type="button"
				onClick={() => onOpen(article)}
				className="flex min-w-0 flex-1 items-start gap-3 py-3 text-left outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
			>
				<Icon
					aria-hidden
					className="mt-0.5 size-4 shrink-0 text-muted-foreground"
				/>
				<span className="grid min-w-0 flex-1 gap-1">
					<span className="truncate font-medium">{article.title}</span>
					{article.kind === 'document' && article.body ? (
						<span className="line-clamp-1 text-[13px] text-muted-foreground">
							{article.body}
						</span>
					) : null}
					{article.tags.length > 0 ? (
						<span className="flex flex-wrap gap-1.5">
							{article.tags.map((tag) => (
								<span
									key={tag}
									className="rounded-full bg-muted px-2 py-0.5 text-xs"
								>
									{tag}
								</span>
							))}
						</span>
					) : null}
				</span>
				<span className="shrink-0 text-right text-muted-foreground text-xs">
					<span className="block">
						{article.file
							? `${fileTypeLabel(article.file.mime_type)} · ${formatSize(article.file.size_bytes)}`
							: 'Document'}
					</span>
					<span className="block">{formatDate(article.updated_at)}</span>
				</span>
			</button>
		</li>
	)
}

function Breadcrumbs({
	path,
	onOpenFolder,
	onDropArticles,
}: {
	path: readonly KnowledgeFolder[]
	onOpenFolder: (folderId: string | null) => void
	onDropArticles?: (folderId: string | null, articleIds: string[]) => void
}) {
	const crumbs: { id: string | null; name: string }[] = [
		{ id: null, name: 'Drive' },
		...path.map((folder) => ({ id: folder.id, name: folder.name })),
	]
	return (
		<nav aria-label="Emplacement">
			<ol className="flex flex-wrap items-center gap-1 text-sm">
				{crumbs.map((crumb, index) => {
					const isLast = index === crumbs.length - 1
					return (
						<li key={crumb.id ?? 'root'} className="flex items-center gap-1">
							{index > 0 ? (
								<ChevronRight
									aria-hidden
									className="size-3.5 text-muted-foreground"
								/>
							) : null}
							{isLast ? (
								<span aria-current="page" className="px-1.5 py-0.5 font-medium">
									{crumb.name}
								</span>
							) : (
								<DropRow folderId={crumb.id} onDropArticles={onDropArticles}>
									<button
										type="button"
										className="rounded px-1.5 py-0.5 text-muted-foreground hover:bg-muted hover:text-foreground"
										onClick={() => onOpenFolder(crumb.id)}
									>
										{crumb.name}
									</button>
								</DropRow>
							)}
						</li>
					)
				})}
			</ol>
		</nav>
	)
}

export function KnowledgePage({
	folders,
	currentFolderId,
	path,
	subfolders,
	expanded,
	articles,
	selected,
	canManage,
	canCreateFolder,
	uploads,
	isUploading,
	isLoading,
	error,
	hasFilters,
	page,
	hasNext,
	total,
	q,
	tagInput,
	onOpenFolder,
	onToggleFolder,
	onNewFolder,
	onRenameFolder,
	onMoveFolder,
	onDeleteFolder,
	onDropArticles,
	onToggleSelected,
	onClearSelection,
	onMoveSelected,
	onFiles,
	onClearUploads,
	onNewDocument,
	isCreatingDocument,
	onQChange,
	onTagInputChange,
	onPageChange,
	onOpen,
}: KnowledgePageProps) {
	const baseId = useId()
	const dropArticles = canManage ? onDropArticles : undefined
	const here = path.at(-1)?.name ?? 'Drive'
	const showFolders = !hasFilters && subfolders.length > 0
	const isEmpty = articles.length === 0 && !showFolders

	return (
		<Page>
			<PageHeader
				title="Drive"
				description="Les devis, tarifs, conditions et documents de l’entreprise, rangés en dossiers. L’assistant les retrouve pour créer des devis ou des projets."
				actions={
					canManage ? (
						<div className="flex flex-wrap gap-2">
							<Button
								type="button"
								variant="outline"
								disabled={!canCreateFolder}
								title={
									canCreateFolder
										? undefined
										: 'Les dossiers s’imbriquent sur 10 niveaux au plus.'
								}
								onClick={onNewFolder}
							>
								<FolderPlus aria-hidden />
								Nouveau dossier
							</Button>
							<Button
								type="button"
								disabled={isCreatingDocument}
								onClick={onNewDocument}
							>
								{isCreatingDocument ? (
									<Loader2 aria-hidden className="animate-spin" />
								) : (
									<Plus aria-hidden />
								)}
								Nouveau document
							</Button>
						</div>
					) : null
				}
			/>
			<div className="grid gap-6 md:grid-cols-[minmax(0,15rem)_minmax(0,1fr)]">
				<aside className="md:sticky md:top-4 md:self-start">
					<div className="rounded-lg border bg-card p-2">
						<FolderTree
							folders={folders}
							selectedId={currentFolderId}
							expanded={expanded}
							onSelect={onOpenFolder}
							onToggle={onToggleFolder}
							onDropArticles={dropArticles}
						/>
					</div>
				</aside>
				<div className="grid min-w-0 content-start gap-5">
					<Breadcrumbs
						path={path}
						onOpenFolder={onOpenFolder}
						onDropArticles={dropArticles}
					/>
					{canManage ? (
						<>
							<FileDropZone
								accept={ACCEPT}
								label={`Dépose des documents dans « ${here} »`}
								hint="PDF, Word, Excel, image ou texte, 25 Mo au plus, plusieurs à la fois"
								isUploading={isUploading}
								onFiles={onFiles}
							/>
							<UploadList
								items={uploads}
								isBusy={isUploading}
								onClear={onClearUploads}
							/>
						</>
					) : null}
					<search
						aria-label="Filtres du drive"
						className="flex flex-wrap items-end gap-3"
					>
						<div className="grid min-w-52 flex-1 gap-1">
							<Label htmlFor={`${baseId}-q`} className="text-muted-foreground">
								{currentFolderId === null
									? 'Recherche dans tout le drive'
									: `Recherche dans « ${here} » et ses sous-dossiers`}
							</Label>
							<div className="relative">
								<Search
									aria-hidden
									className="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
								/>
								<Input
									id={`${baseId}-q`}
									type="search"
									value={q}
									placeholder="Titre, texte d’un document"
									autoComplete="off"
									className="pl-8"
									onChange={(event) => onQChange(event.target.value)}
								/>
							</div>
						</div>
						<div className="grid min-w-44 gap-1">
							<Label
								htmlFor={`${baseId}-tags`}
								className="text-muted-foreground"
							>
								Étiquettes
							</Label>
							<Input
								id={`${baseId}-tags`}
								value={tagInput}
								placeholder="devis, cgv"
								autoComplete="off"
								onChange={(event) => onTagInputChange(event.target.value)}
							/>
						</div>
					</search>
					{selected.size > 0 ? (
						<div
							role="toolbar"
							aria-label="Sélection"
							className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/40 px-3 py-2"
						>
							<span className="text-sm">
								{selected.size === 1
									? '1 élément sélectionné'
									: `${selected.size} éléments sélectionnés`}
							</span>
							<span className="flex-1" />
							<Button type="button" size="sm" onClick={onMoveSelected}>
								<FolderInput aria-hidden />
								Déplacer vers…
							</Button>
							<Button
								type="button"
								size="sm"
								variant="ghost"
								onClick={onClearSelection}
							>
								<X aria-hidden />
								Annuler
							</Button>
						</div>
					) : null}
					{error ? (
						<div
							role="alert"
							className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-destructive text-sm"
						>
							{error}
						</div>
					) : null}
					{isLoading ? (
						<div className="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
							<Loader2 aria-hidden className="size-4 animate-spin" />
							Chargement…
						</div>
					) : isEmpty ? (
						<EmptyState
							title={
								hasFilters
									? 'Aucun résultat'
									: currentFolderId === null
										? 'Le drive est vide'
										: 'Ce dossier est vide'
							}
							hint={
								hasFilters
									? 'Modifie ou retire un filtre pour voir plus de documents.'
									: canManage
										? 'Dépose un fichier, écris un document ou crée un dossier.'
										: 'Personne n’a encore rien ajouté ici.'
							}
						/>
					) : (
						<>
							<ul className="divide-y rounded-lg border bg-card">
								{showFolders
									? subfolders.map((folder) => (
											<FolderRow
												key={folder.id}
												folder={folder}
												canManage={canManage}
												onOpen={() => onOpenFolder(folder.id)}
												onRename={() => onRenameFolder(folder)}
												onMove={() => onMoveFolder(folder)}
												onDelete={() => onDeleteFolder(folder)}
												onDropArticles={dropArticles}
											/>
										))
									: null}
								{articles.map((article) => (
									<ArticleRow
										key={article.id}
										article={article}
										canManage={canManage}
										isSelected={selected.has(article.id)}
										selected={selected}
										onToggle={() => onToggleSelected(article.id)}
										onOpen={onOpen}
									/>
								))}
							</ul>
							{articles.length > 0 ? (
								<nav
									aria-label="Pagination"
									className="flex items-center justify-between gap-2"
								>
									<p className="text-muted-foreground text-sm">
										{total === null
											? `Page ${page}`
											: `${total} éléments, page ${page}`}
									</p>
									<div className="flex gap-2">
										<Button
											type="button"
											variant="outline"
											size="sm"
											disabled={page <= 1}
											onClick={() => onPageChange(page - 1)}
										>
											<ChevronLeft aria-hidden />
											Précédente
										</Button>
										<Button
											type="button"
											variant="outline"
											size="sm"
											disabled={!hasNext}
											onClick={() => onPageChange(page + 1)}
										>
											Suivante
											<ChevronRight aria-hidden />
										</Button>
									</div>
								</nav>
							) : null}
						</>
					)}
				</div>
			</div>
		</Page>
	)
}

import {
	ChevronLeft,
	ChevronRight,
	FileText,
	Loader2,
	Plus,
	Search,
	StickyNote,
} from 'lucide-react'
import { useId } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { KnowledgeArticle } from '#/hooks/use-knowledge'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import { ACCEPT, fileTypeLabel, formatSize } from '#/pages/knowledge/lib/files'
import type { UploadItem } from '#/pages/photo-library/lib/uploads'
import { UploadList } from '#/pages/photo-library/ui/upload-list'
import { Page, PageHeader } from '#/pages/projects/ui/page'

interface KnowledgePageProps {
	articles: readonly KnowledgeArticle[]
	canManage: boolean
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
	onFiles: (files: File[]) => void
	onClearUploads: () => void
	onNewNote: () => void
	onQChange: (q: string) => void
	onTagInputChange: (tagInput: string) => void
	onPageChange: (page: number) => void
	onOpen: (article: KnowledgeArticle) => void
}

function formatDate(iso: string): string {
	return new Date(iso).toLocaleDateString('fr-FR')
}

function ArticleRow({
	article,
	onOpen,
}: {
	article: KnowledgeArticle
	onOpen: (article: KnowledgeArticle) => void
}) {
	const Icon = article.kind === 'note' ? StickyNote : FileText
	return (
		<li>
			<button
				type="button"
				onClick={() => onOpen(article)}
				className="flex w-full items-start gap-3 px-4 py-3 text-left outline-none hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50"
			>
				<Icon
					aria-hidden
					className="mt-0.5 size-4 shrink-0 text-muted-foreground"
				/>
				<span className="grid min-w-0 flex-1 gap-1">
					<span className="truncate font-medium">{article.title}</span>
					{article.kind === 'note' && article.body ? (
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
							: 'Note'}
					</span>
					<span className="block">{formatDate(article.updated_at)}</span>
				</span>
			</button>
		</li>
	)
}

export function KnowledgePage({
	articles,
	canManage,
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
	onFiles,
	onClearUploads,
	onNewNote,
	onQChange,
	onTagInputChange,
	onPageChange,
	onOpen,
}: KnowledgePageProps) {
	const baseId = useId()
	return (
		<Page>
			<PageHeader
				title="Base de connaissances"
				description="Les devis, tarifs, conditions et notes de l’entreprise. L’assistant les retrouve pour créer des devis ou des projets."
				actions={
					canManage ? (
						<Button type="button" onClick={onNewNote}>
							<Plus aria-hidden />
							Nouvelle note
						</Button>
					) : null
				}
			/>
			<div className="grid gap-5">
				{canManage ? (
					<>
						<FileDropZone
							accept={ACCEPT}
							label="Dépose des documents ici"
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
					aria-label="Filtres de la base de connaissances"
					className="flex flex-wrap items-end gap-3"
				>
					<div className="grid min-w-52 flex-1 gap-1">
						<Label htmlFor={`${baseId}-q`} className="text-muted-foreground">
							Recherche
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
								placeholder="Titre, note, texte d’un document"
								autoComplete="off"
								className="pl-8"
								onChange={(event) => onQChange(event.target.value)}
							/>
						</div>
					</div>
					<div className="grid min-w-44 gap-1">
						<Label htmlFor={`${baseId}-tags`} className="text-muted-foreground">
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
				) : articles.length === 0 ? (
					<EmptyState
						title={
							hasFilters
								? 'Aucun résultat'
								: 'La base de connaissances est vide'
						}
						hint={
							hasFilters
								? 'Modifie ou retire un filtre pour voir plus de documents.'
								: canManage
									? 'Dépose un premier document ou écris une note.'
									: 'Personne n’a encore rien ajouté.'
						}
					/>
				) : (
					<>
						<ul className="divide-y rounded-lg border bg-card">
							{articles.map((article) => (
								<ArticleRow
									key={article.id}
									article={article}
									onOpen={onOpen}
								/>
							))}
						</ul>
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
					</>
				)}
			</div>
		</Page>
	)
}

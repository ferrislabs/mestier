import { FileText, Link2, Loader2, Search, StickyNote, X } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { KnowledgeArticle } from '#/hooks/use-knowledge'
import { fileTypeLabel, formatSize } from '#/pages/knowledge/lib/files'

function ArticleIcon({ article }: { article: KnowledgeArticle }) {
	const Icon = article.kind === 'note' ? StickyNote : FileText
	return <Icon aria-hidden className="size-4 shrink-0 text-muted-foreground" />
}

function kindLabel(article: KnowledgeArticle): string {
	return article.file
		? `${fileTypeLabel(article.file.mime_type)} · ${formatSize(article.file.size_bytes)}`
		: 'Note'
}

interface LinkedKnowledgeProps {
	articles: readonly KnowledgeArticle[]
	isLoading: boolean
	error: string | null
	canManage: boolean
	onOpen: (article: KnowledgeArticle) => void
	onUnlink: (article: KnowledgeArticle) => void
	onAttach: () => void
}

export function LinkedKnowledge({
	articles,
	isLoading,
	error,
	canManage,
	onOpen,
	onUnlink,
	onAttach,
}: LinkedKnowledgeProps) {
	return (
		<section
			aria-label="Documents liés"
			className="mx-auto grid w-full max-w-6xl gap-3 px-6 pt-2 pb-10 lg:px-10"
		>
			<div className="flex items-center justify-between gap-3">
				<h2 className="font-semibold text-base">
					Documents liés{articles.length > 0 ? ` (${articles.length})` : ''}
				</h2>
				{canManage ? (
					<Button type="button" variant="outline" size="sm" onClick={onAttach}>
						<Link2 aria-hidden />
						Rattacher un document
					</Button>
				) : null}
			</div>
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			{isLoading ? (
				<div className="flex items-center gap-2 text-muted-foreground text-sm">
					<Loader2 aria-hidden className="size-4 animate-spin" />
					Chargement…
				</div>
			) : articles.length === 0 ? (
				<p className="text-[13px] text-muted-foreground">
					Aucun document ni note rattaché.
				</p>
			) : (
				<ul className="divide-y rounded-lg border bg-card">
					{articles.map((article) => (
						<li key={article.id} className="flex items-center gap-2 pr-2">
							<button
								type="button"
								onClick={() => onOpen(article)}
								className="flex min-w-0 flex-1 items-center gap-3 px-4 py-2.5 text-left outline-none hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50"
							>
								<ArticleIcon article={article} />
								<span className="min-w-0 flex-1 truncate text-sm">
									{article.title}
								</span>
								<span className="shrink-0 text-muted-foreground text-xs">
									{kindLabel(article)}
								</span>
							</button>
							{canManage ? (
								<Button
									type="button"
									variant="ghost"
									size="icon"
									aria-label={`Détacher ${article.title}`}
									onClick={() => onUnlink(article)}
								>
									<X aria-hidden />
								</Button>
							) : null}
						</li>
					))}
				</ul>
			)}
		</section>
	)
}

interface AttachDialogProps {
	open: boolean
	query: string
	candidates: readonly KnowledgeArticle[]
	isLoading: boolean
	isAttaching: boolean
	error: string | null
	onQueryChange: (query: string) => void
	onPick: (article: KnowledgeArticle) => void
	onOpenChange: (open: boolean) => void
}

export function AttachKnowledgeDialog({
	open,
	query,
	candidates,
	isLoading,
	isAttaching,
	error,
	onQueryChange,
	onPick,
	onOpenChange,
}: AttachDialogProps) {
	const id = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>Rattacher un document</DialogTitle>
					<DialogDescription>
						Choisis un document ou une note de la base de connaissances.
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-1">
					<Label htmlFor={id} className="text-muted-foreground">
						Recherche
					</Label>
					<div className="relative">
						<Search
							aria-hidden
							className="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
						/>
						<Input
							id={id}
							type="search"
							value={query}
							placeholder="Titre, note, texte d’un document"
							autoComplete="off"
							className="pl-8"
							onChange={(event) => onQueryChange(event.target.value)}
						/>
					</div>
				</div>
				{error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
				{isLoading ? (
					<div className="flex items-center gap-2 text-muted-foreground text-sm">
						<Loader2 aria-hidden className="size-4 animate-spin" />
						Recherche…
					</div>
				) : candidates.length === 0 ? (
					<p className="text-[13px] text-muted-foreground">
						Rien à rattacher : tout est déjà lié, ou rien ne correspond.
					</p>
				) : (
					<ul className="max-h-80 divide-y overflow-y-auto rounded-md border">
						{candidates.map((article) => (
							<li key={article.id}>
								<button
									type="button"
									disabled={isAttaching}
									onClick={() => onPick(article)}
									className="flex w-full items-center gap-3 px-3 py-2 text-left outline-none hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50"
								>
									<ArticleIcon article={article} />
									<span className="min-w-0 flex-1 truncate text-sm">
										{article.title}
									</span>
									<span className="shrink-0 text-muted-foreground text-xs">
										{kindLabel(article)}
									</span>
								</button>
							</li>
						))}
					</ul>
				)}
			</DialogContent>
		</Dialog>
	)
}

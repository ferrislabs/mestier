import type { ColumnDef, Table } from '@tanstack/react-table'
import {
	ArrowLeft,
	ChevronLeft,
	ChevronRight,
	ImageOff,
	Trash2,
} from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { type AnswerView, answerView } from '#/pages/survey/lib/answer-format'
import { questionKindEntry } from '#/pages/survey/lib/question-kinds'
import { formatDateTime } from '#/pages/survey/lib/survey-format'
import type { QuestionNode, SurveyResponse } from '#/pages/survey/types'
import { ResponseStatusBadge } from '#/pages/survey/ui/survey-badges'
import { SurveyDataTable } from '#/pages/survey/ui/survey-data-table'
import { ErrorBanner, LoadingCard } from '#/pages/survey/ui/surveys-list'

export interface ResponseRow {
	response: SurveyResponse
	respondent: string
}

export function responseColumns(): ColumnDef<ResponseRow>[] {
	return [
		{
			id: 'status',
			header: 'Statut',
			cell: ({ row }) => (
				<ResponseStatusBadge status={row.original.response.status} />
			),
		},
		{
			id: 'respondent',
			header: 'Répondant',
			cell: ({ row }) => (
				<span className="font-medium">{row.original.respondent}</span>
			),
		},
		{
			id: 'started',
			header: 'Commencée le',
			cell: ({ row }) => formatDateTime(row.original.response.started_at),
		},
		{
			id: 'completed',
			header: 'Complétée le',
			cell: ({ row }) => formatDateTime(row.original.response.completed_at),
		},
	]
}

export function SurveyResponsesList({
	title,
	table,
	isEmpty,
	isLoading,
	error,
	page,
	pageCount,
	total,
	onBack,
	onPage,
	onOpen,
}: {
	title: string
	table: Table<ResponseRow>
	isEmpty: boolean
	isLoading: boolean
	error: string | null
	page: number
	pageCount: number | null
	total: number | null
	onBack: () => void
	onPage: (page: number) => void
	onOpen: (row: ResponseRow) => void
}) {
	return (
		<PageShell>
			<PageHeader
				leading={
					<Button
						variant="ghost"
						size="icon-sm"
						aria-label="Retour au questionnaire"
						onClick={onBack}
					>
						<ArrowLeft aria-hidden />
					</Button>
				}
				eyebrow={title}
				title="Réponses"
				description={
					total == null
						? 'Les réponses reçues, en cours et complètes.'
						: `${total} réponse${total > 1 ? 's' : ''} reçue${total > 1 ? 's' : ''}.`
				}
			/>
			<ErrorBanner message={error} />
			{isLoading ? (
				<LoadingCard label="Chargement des réponses…" />
			) : isEmpty ? (
				<EmptyState
					title="Aucune réponse pour le moment"
					hint="Les réponses apparaissent ici dès que le questionnaire est ouvert et qu’un répondant commence."
				/>
			) : (
				<>
					<SurveyDataTable table={table} label="Réponses" onRowClick={onOpen} />
					<nav
						aria-label="Pagination des réponses"
						className="flex items-center justify-end gap-2 text-sm"
					>
						<Button
							variant="outline"
							size="sm"
							disabled={page <= 1}
							onClick={() => onPage(page - 1)}
						>
							<ChevronLeft aria-hidden />
							Précédente
						</Button>
						<span className="text-muted-foreground">
							Page {page}
							{pageCount ? ` sur ${pageCount}` : ''}
						</span>
						<Button
							variant="outline"
							size="sm"
							disabled={pageCount == null || page >= pageCount}
							onClick={() => onPage(page + 1)}
						>
							Suivante
							<ChevronRight aria-hidden />
						</Button>
					</nav>
				</>
			)}
		</PageShell>
	)
}

function AnswerValue({ view }: { view: AnswerView }) {
	switch (view.type) {
		case 'empty':
			return <span className="text-muted-foreground italic">Sans réponse</span>
		case 'text':
			return <span className="whitespace-pre-wrap">{view.text}</span>
		case 'choices':
			return (
				<ul className="flex flex-wrap gap-1">
					{view.choices.map((choice) => (
						<li
							key={choice}
							className="rounded-md bg-muted px-2 py-0.5 text-xs"
						>
							{choice}
						</li>
					))}
				</ul>
			)
		case 'photos':
			return (
				<ul className="grid gap-1">
					{view.keys.map((key) => (
						<li
							key={key}
							className="flex items-center gap-2 break-all text-muted-foreground text-xs"
						>
							<ImageOff className="size-3.5 shrink-0" aria-hidden />
							{key}
						</li>
					))}
				</ul>
			)
	}
}

export function ResponseDetailSheet({
	row,
	path,
	canManage,
	onClose,
	onDelete,
}: {
	row: ResponseRow | null
	path: readonly QuestionNode[]
	canManage: boolean
	onClose: () => void
	onDelete: () => void
}) {
	return (
		<Sheet
			open={row !== null}
			onOpenChange={(open) => (open ? null : onClose())}
		>
			<SheetContent className="w-full overflow-y-auto sm:max-w-lg">
				{row ? (
					<>
						<SheetHeader>
							<SheetTitle>{row.respondent}</SheetTitle>
							<SheetDescription className="flex flex-wrap items-center gap-2">
								<ResponseStatusBadge status={row.response.status} />
								<span>
									Commencée le {formatDateTime(row.response.started_at)}
								</span>
								{row.response.completed_at ? (
									<span>
										· complétée le {formatDateTime(row.response.completed_at)}
									</span>
								) : null}
							</SheetDescription>
						</SheetHeader>
						<ol className="grid gap-4 px-4" aria-label="Réponses du parcours">
							{path.map((question) => {
								const entry = questionKindEntry(question.kind)
								const Icon = entry.icon
								return (
									<li key={question.id} className="grid gap-1">
										<p className="flex items-center gap-1.5 text-muted-foreground text-xs">
											<Icon className="size-3.5" aria-hidden />
											{entry.title}
										</p>
										<p className="font-medium text-sm">
											{question.label || 'Question sans libellé'}
										</p>
										<div className="text-sm">
											<AnswerValue
												view={answerView(row.response.answers[question.id])}
											/>
										</div>
									</li>
								)
							})}
						</ol>
						{path.some((question) => question.kind === 'PHOTO') ? (
							<p className="px-4 text-muted-foreground text-xs">
								Les photos sont listées par leur clé de stockage : leur aperçu
								n’est pas encore disponible ici.
							</p>
						) : null}
						{canManage ? (
							<div className="px-4 pb-6">
								<Button variant="outline" onClick={onDelete}>
									<Trash2 aria-hidden />
									Supprimer la réponse
								</Button>
							</div>
						) : null}
					</>
				) : null}
			</SheetContent>
		</Sheet>
	)
}

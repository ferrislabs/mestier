import type { Table } from '@tanstack/react-table'
import { Loader2, Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { SurveyRow } from '#/pages/survey/ui/survey-columns'
import { SurveyDataTable } from '#/pages/survey/ui/survey-data-table'

export function ErrorBanner({ message }: { message: string | null }) {
	if (!message) return null
	return (
		<div
			role="alert"
			className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
		>
			{message}
		</div>
	)
}

export function LoadingCard({ label }: { label: string }) {
	return (
		<div className="flex items-center justify-center gap-3 rounded-lg border bg-card px-6 py-12 text-sm text-muted-foreground">
			<Loader2 className="size-5 animate-spin" aria-hidden />
			{label}
		</div>
	)
}

export function SurveysList({
	table,
	isEmpty,
	isLoading,
	error,
	canManage,
	onCreate,
}: {
	table: Table<SurveyRow>
	isEmpty: boolean
	isLoading: boolean
	error: string | null
	canManage: boolean
	onCreate: () => void
}) {
	const createButton = canManage ? (
		<Button onClick={onCreate}>
			<Plus aria-hidden />
			Nouveau questionnaire
		</Button>
	) : null
	return (
		<PageShell>
			<PageHeader
				title="Questionnaires"
				description="Construisez un questionnaire, ouvrez-le à votre audience et lisez les réponses reçues."
				actions={createButton}
			/>
			<ErrorBanner message={error} />
			{isLoading ? (
				<LoadingCard label="Chargement des questionnaires…" />
			) : isEmpty ? (
				<EmptyState
					title="Aucun questionnaire pour le moment"
					hint="Un questionnaire commence en brouillon : vous l’ouvrez quand il est prêt."
				>
					{createButton}
				</EmptyState>
			) : (
				<SurveyDataTable table={table} label="Questionnaires" />
			)}
		</PageShell>
	)
}

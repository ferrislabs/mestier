import type { Table } from '@tanstack/react-table'
import { Loader2, Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Campaign } from '#/pages/survey/types'
import { NATIVE_SELECT } from '#/pages/survey/ui/native-select'
import type { SurveyRow } from '#/pages/survey/ui/survey-columns'
import { SurveyDataTable } from '#/pages/survey/ui/survey-data-table'

export const INDEPENDENT = '__independent__'

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

export interface LoadMore {
	shown: number
	total: number
	hasMore: boolean
	loading: boolean
	onLoadMore: () => void
}

export function SurveysList({
	table,
	isEmpty,
	isLoading,
	error,
	canManage,
	campaigns,
	campaignFilter,
	onCampaignFilter,
	more,
	onCreate,
}: {
	table: Table<SurveyRow>
	isEmpty: boolean
	isLoading: boolean
	error: string | null
	canManage: boolean
	campaigns: readonly Campaign[]
	campaignFilter: string
	onCampaignFilter: (campaignId: string) => void
	more: LoadMore | null
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
			{campaigns.length > 0 ? (
				<div className="flex flex-wrap items-center gap-2">
					<label htmlFor="surveys-campaign-filter" className="text-sm">
						Campagne
					</label>
					<select
						id="surveys-campaign-filter"
						className={`${NATIVE_SELECT} w-auto min-w-48`}
						value={campaignFilter}
						onChange={(event) => onCampaignFilter(event.target.value)}
					>
						<option value="">Toutes les campagnes</option>
						<option value={INDEPENDENT}>Sans campagne</option>
						{campaigns.map((campaign) => (
							<option key={campaign.id} value={campaign.id}>
								{campaign.name}
							</option>
						))}
					</select>
				</div>
			) : null}
			{isLoading ? (
				<LoadingCard label="Chargement des questionnaires…" />
			) : isEmpty && campaignFilter !== '' ? (
				<EmptyState
					title="Aucun questionnaire pour cette campagne"
					hint="Rattachez un questionnaire à la campagne depuis ses réglages."
				/>
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
			{!isLoading && !isEmpty && more ? (
				<div className="flex flex-wrap items-center justify-between gap-2 text-muted-foreground text-sm">
					<span>
						{more.shown} sur {more.total}
					</span>
					{more.hasMore ? (
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={more.loading}
							onClick={more.onLoadMore}
						>
							Voir plus
						</Button>
					) : null}
				</div>
			) : null}
		</PageShell>
	)
}

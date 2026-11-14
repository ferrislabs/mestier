import type { ColumnDef, Table } from '@tanstack/react-table'
import { Pencil, Plus, Trash2 } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { Notice } from '#/pages/customers/ui/notice'
import { formatWindow } from '#/pages/survey/lib/survey-format'
import type { Campaign } from '#/pages/survey/types'
import { CampaignStatusBadge } from '#/pages/survey/ui/survey-badges'
import { SurveyDataTable } from '#/pages/survey/ui/survey-data-table'
import { ErrorBanner, LoadingCard } from '#/pages/survey/ui/surveys-list'

export interface CampaignRow {
	campaign: Campaign
	surveyCount: number
}

export function campaignColumns({
	canManage,
	onEdit,
	onDelete,
}: {
	canManage: boolean
	onEdit: (row: CampaignRow) => void
	onDelete: (row: CampaignRow) => void
}): ColumnDef<CampaignRow>[] {
	return [
		{
			id: 'name',
			header: 'Campagne',
			cell: ({ row }) => (
				<button
					type="button"
					className="max-w-full truncate text-left font-medium hover:underline"
					onClick={() => onEdit(row.original)}
				>
					{row.original.campaign.name}
				</button>
			),
		},
		{
			id: 'window',
			header: 'Fenêtre',
			cell: ({ row }) =>
				formatWindow(
					row.original.campaign.starts_at,
					row.original.campaign.ends_at,
				),
		},
		{
			id: 'status',
			header: 'Statut',
			cell: ({ row }) => (
				<CampaignStatusBadge status={row.original.campaign.status} />
			),
		},
		{
			id: 'surveys',
			header: 'Questionnaires',
			cell: ({ row }) => row.original.surveyCount,
		},
		{
			id: 'actions',
			header: () => <span className="sr-only">Actions</span>,
			cell: ({ row }) => (
				<div className="flex justify-end gap-1">
					<Button
						variant="ghost"
						size="icon-sm"
						aria-label={`${canManage ? 'Modifier' : 'Voir'} « ${row.original.campaign.name} »`}
						onClick={() => onEdit(row.original)}
					>
						<Pencil aria-hidden />
					</Button>
					{canManage ? (
						<Button
							variant="ghost"
							size="icon-sm"
							aria-label={`Supprimer « ${row.original.campaign.name} »`}
							onClick={() => onDelete(row.original)}
						>
							<Trash2 aria-hidden />
						</Button>
					) : null}
				</div>
			),
		},
	]
}

export function CampaignsList({
	table,
	isEmpty,
	isLoading,
	error,
	canManage,
	notice,
	onDismissNotice,
	onCreate,
}: {
	table: Table<CampaignRow>
	isEmpty: boolean
	isLoading: boolean
	error: string | null
	canManage: boolean
	notice: string | null
	onDismissNotice: () => void
	onCreate: () => void
}) {
	const createButton = canManage ? (
		<Button onClick={onCreate}>
			<Plus aria-hidden />
			Nouvelle campagne
		</Button>
	) : null
	return (
		<PageShell>
			<PageHeader
				title="Campagnes"
				description="Une campagne regroupe des questionnaires et fixe la fenêtre pendant laquelle leurs réponses sont acceptées."
				actions={createButton}
			/>
			<Notice message={notice} onDismiss={onDismissNotice} />
			<ErrorBanner message={error} />
			{isLoading ? (
				<LoadingCard label="Chargement des campagnes…" />
			) : isEmpty ? (
				<EmptyState
					title="Aucune campagne"
					hint="Un questionnaire peut rester indépendant : une campagne n’est utile que pour une fenêtre de réponses."
				>
					{createButton}
				</EmptyState>
			) : (
				<SurveyDataTable table={table} label="Campagnes" />
			)}
		</PageShell>
	)
}

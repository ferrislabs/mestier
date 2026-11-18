import type { ColumnDef } from '@tanstack/react-table'
import {
	Copy,
	CopyPlus,
	DoorClosed,
	DoorOpen,
	Inbox,
	MoreHorizontal,
	Pencil,
	Trash2,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { formatDateTime } from '#/pages/survey/lib/survey-format'
import type { Survey } from '#/pages/survey/types'
import {
	SurveyStatusBadge,
	VisibilityBadge,
} from '#/pages/survey/ui/survey-badges'

export interface SurveyRow {
	survey: Survey
	campaignNames: string[]
}

export type SurveyAction =
	| 'edit'
	| 'responses'
	| 'copy-link'
	| 'duplicate'
	| 'open'
	| 'close'
	| 'delete'

export function availableActions(
	survey: Survey,
	canManage: boolean,
): SurveyAction[] {
	const actions: SurveyAction[] = ['edit', 'responses']
	if (
		survey.status === 'OPENED' &&
		survey.visibility === 'PUBLIC' &&
		survey.public_slug
	) {
		actions.push('copy-link')
	}
	if (!canManage) return actions
	actions.push('duplicate')
	if (survey.status === 'DRAFT') actions.push('open')
	if (survey.status === 'OPENED') actions.push('close')
	actions.push('delete')
	return actions
}

const ACTION_LABELS: Record<SurveyAction, string> = {
	edit: 'Ouvrir l’éditeur',
	responses: 'Voir les réponses',
	'copy-link': 'Copier le lien public',
	duplicate: 'Dupliquer',
	open: 'Ouvrir aux réponses…',
	close: 'Fermer…',
	delete: 'Supprimer…',
}

const ACTION_ICONS = {
	edit: Pencil,
	responses: Inbox,
	'copy-link': Copy,
	duplicate: CopyPlus,
	open: DoorOpen,
	close: DoorClosed,
	delete: Trash2,
} satisfies Record<SurveyAction, unknown>

function RowActions({
	row,
	canManage,
	duplicating,
	onAction,
}: {
	row: SurveyRow
	canManage: boolean
	duplicating: boolean
	onAction: (action: SurveyAction, row: SurveyRow) => void
}) {
	const actions = availableActions(row.survey, canManage)
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label={`Actions sur « ${row.survey.title} »`}
					onClick={(event) => event.stopPropagation()}
				>
					<MoreHorizontal aria-hidden />
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end">
				{actions.map((action) => {
					const Icon = ACTION_ICONS[action]
					return [
						action === 'delete' ? (
							<DropdownMenuSeparator key="separator" />
						) : null,
						<DropdownMenuItem
							key={action}
							variant={action === 'delete' ? 'destructive' : 'default'}
							disabled={action === 'duplicate' && duplicating}
							onSelect={() => onAction(action, row)}
						>
							<Icon aria-hidden />
							{ACTION_LABELS[action]}
						</DropdownMenuItem>,
					]
				})}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

export function surveyColumns({
	canManage,
	duplicating,
	onAction,
}: {
	canManage: boolean
	duplicating: boolean
	onAction: (action: SurveyAction, row: SurveyRow) => void
}): ColumnDef<SurveyRow>[] {
	return [
		{
			id: 'title',
			header: 'Questionnaire',
			cell: ({ row }) => (
				<button
					type="button"
					className="max-w-full truncate text-left font-medium hover:underline"
					onClick={() => onAction('edit', row.original)}
				>
					{row.original.survey.title}
				</button>
			),
		},
		{
			id: 'visibility',
			header: 'Visibilité',
			cell: ({ row }) => (
				<VisibilityBadge visibility={row.original.survey.visibility} />
			),
		},
		{
			id: 'status',
			header: 'Statut',
			cell: ({ row }) => (
				<SurveyStatusBadge status={row.original.survey.status} />
			),
		},
		{
			id: 'campaign',
			header: 'Campagnes',
			cell: ({ row }) =>
				row.original.campaignNames.length > 0 ? (
					<ul className="flex flex-wrap gap-1">
						{row.original.campaignNames.map((name) => (
							<li
								key={name}
								className="rounded-md bg-muted px-2 py-0.5 text-xs"
							>
								{name}
							</li>
						))}
					</ul>
				) : (
					<span className="text-muted-foreground">Indépendant</span>
				),
		},
		{
			id: 'opened',
			header: 'Ouvert le',
			cell: ({ row }) => formatDateTime(row.original.survey.opened_at),
		},
		{
			id: 'closed',
			header: 'Fermé le',
			cell: ({ row }) => formatDateTime(row.original.survey.closed_at),
		},
		{
			id: 'actions',
			header: () => <span className="sr-only">Actions</span>,
			cell: ({ row }) => (
				<div className="flex justify-end">
					<RowActions
						row={row.original}
						canManage={canManage}
						duplicating={duplicating}
						onAction={onAction}
					/>
				</div>
			),
		},
	]
}

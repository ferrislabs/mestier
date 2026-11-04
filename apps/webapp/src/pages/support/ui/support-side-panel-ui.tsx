import { Label } from '#/components/ui/label'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import {
	supportStatusLabel,
	supportStatusTone,
} from '#/pages/support/lib/columns'
import type { SupportSidePanelUIProps } from '#/pages/support/types'

const SELECT_CLASS_NAME =
	'h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm disabled:opacity-50'

const ROW_LABEL_CLASS_NAME = 'text-xs font-semibold text-muted-foreground'

export function SupportSidePanelUI({
	ticket,
	canManage,
	statusOptions,
	assigneeLabel,
	assignees,
	isUpdating,
	error,
	onChangeStatus,
	onChangeAssignee,
}: SupportSidePanelUIProps) {
	const statusLabel =
		statusOptions.find((option) => option.status === ticket.status)?.label ??
		supportStatusLabel(ticket.status)

	function handleStatusChange(value: string) {
		const option = statusOptions.find((entry) => entry.status === value)
		if (option) onChangeStatus(option.status)
	}

	return (
		<SectionCard className="min-w-0 overflow-hidden">
			<SectionHeader title="Détails" />
			<div className="flex flex-col gap-4 p-5">
				<div className="flex flex-col gap-1.5">
					{canManage ? (
						<Label
							htmlFor="support-ticket-status"
							className={ROW_LABEL_CLASS_NAME}
						>
							Statut
						</Label>
					) : (
						<p className={ROW_LABEL_CLASS_NAME}>Statut</p>
					)}
					{canManage ? (
						<select
							id="support-ticket-status"
							className={SELECT_CLASS_NAME}
							disabled={isUpdating}
							value={ticket.status}
							onChange={(event) => handleStatusChange(event.target.value)}
						>
							{statusOptions.map((option) => (
								<option key={option.status} value={option.status}>
									{option.label}
								</option>
							))}
						</select>
					) : (
						<div>
							<StatusBadge tone={supportStatusTone(ticket.status)}>
								{statusLabel}
							</StatusBadge>
						</div>
					)}
				</div>

				<div className="flex flex-col gap-1.5">
					{canManage ? (
						<Label
							htmlFor="support-ticket-assignee"
							className={ROW_LABEL_CLASS_NAME}
						>
							Assigné à
						</Label>
					) : (
						<p className={ROW_LABEL_CLASS_NAME}>Assigné à</p>
					)}
					{canManage ? (
						<select
							id="support-ticket-assignee"
							className={SELECT_CLASS_NAME}
							disabled={isUpdating}
							value={ticket.assigneeId ?? ''}
							onChange={(event) =>
								onChangeAssignee(
									event.target.value === '' ? null : event.target.value,
								)
							}
						>
							<option value="">Non assigné</option>
							{assignees.map((assignee) => (
								<option key={assignee.id} value={assignee.id}>
									{assignee.label}
								</option>
							))}
						</select>
					) : (
						<p className="text-sm">{assigneeLabel}</p>
					)}
				</div>

				{error ? <p className="text-xs text-destructive">{error}</p> : null}

				<div className="flex flex-col gap-1.5">
					<p className={ROW_LABEL_CLASS_NAME}>Badges</p>
					{ticket.badges.length > 0 ? (
						<div className="flex flex-wrap gap-1">
							{ticket.badges.map((badge) => (
								<StatusBadge key={badge.label} tone={badge.tone}>
									{badge.label}
								</StatusBadge>
							))}
						</div>
					) : (
						<p className="text-sm text-muted-foreground">Aucun badge</p>
					)}
				</div>

				<div className="flex flex-col gap-1.5">
					<p className={ROW_LABEL_CLASS_NAME}>Champs</p>
					{ticket.fields.length > 0 ? (
						<dl className="flex flex-col gap-1 text-sm">
							{ticket.fields.map((field) => (
								<div key={field.key} className="flex items-baseline gap-1.5">
									<dt className="shrink-0 text-muted-foreground">
										{field.key}
									</dt>
									<dd className="min-w-0 break-words">{field.value}</dd>
								</div>
							))}
						</dl>
					) : (
						<p className="text-sm text-muted-foreground">Aucun champ</p>
					)}
				</div>

				<div className="flex flex-col gap-1.5">
					<p className={ROW_LABEL_CLASS_NAME}>Créé le</p>
					<p className="text-sm">{ticket.createdAtLabel}</p>
				</div>
			</div>
		</SectionCard>
	)
}

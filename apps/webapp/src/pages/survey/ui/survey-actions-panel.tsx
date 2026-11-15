import {
	Copy,
	CopyPlus,
	DoorClosed,
	DoorOpen,
	Eye,
	Inbox,
	Save,
	Trash2,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Switch } from '#/components/ui/switch'
import type { Survey } from '#/pages/survey/types'
import {
	SurveyStatusBadge,
	VisibilityBadge,
} from '#/pages/survey/ui/survey-badges'
import { SAVE_REASON_ID } from '#/pages/survey/ui/survey-editor-header'

export function SurveyActionsPanel({
	survey,
	title,
	canManage,
	dirty,
	saving,
	duplicating,
	saveBlockedReason,
	problems,
	previewing,
	onPreviewing,
	onResponses,
	onSave,
	onOpen,
	onClose,
	onDuplicate,
	onDelete,
	onCopyLink,
}: {
	survey: Survey
	title: string
	canManage: boolean
	dirty: boolean
	saving: boolean
	duplicating: boolean
	saveBlockedReason: string | null
	problems: ReactNode
	previewing: boolean
	onPreviewing: (previewing: boolean) => void
	onResponses: () => void
	onSave: () => void
	onOpen: () => void
	onClose: () => void
	onDuplicate: () => void
	onDelete: () => void
	onCopyLink: (() => void) | null
}) {
	const blocked = saveBlockedReason !== null
	return (
		<div className="grid gap-3 border-b p-4">
			<h1 className="sr-only">{title}</h1>
			<div className="flex flex-wrap items-center gap-1.5">
				<SurveyStatusBadge status={survey.status} />
				<VisibilityBadge visibility={survey.visibility} />
				{problems}
			</div>
			{dirty ? (
				<p className="text-muted-foreground text-xs">
					Modifications non enregistrées
				</p>
			) : null}
			{canManage && blocked ? (
				<p id={SAVE_REASON_ID} className="text-muted-foreground text-xs">
					{saveBlockedReason}
				</p>
			) : null}
			<label
				htmlFor="survey-preview-switch"
				className="flex items-center justify-between gap-2 text-sm"
			>
				<span className="flex items-center gap-2">
					<Eye className="size-4" aria-hidden />
					Aperçu
				</span>
				<Switch
					id="survey-preview-switch"
					checked={previewing}
					onCheckedChange={onPreviewing}
				/>
			</label>
			{canManage ? (
				<div className="grid gap-2">
					<Button
						variant={dirty ? 'default' : 'outline'}
						disabled={!dirty || saving}
						aria-disabled={blocked || undefined}
						aria-describedby={blocked ? SAVE_REASON_ID : undefined}
						onClick={onSave}
					>
						<Save aria-hidden />
						Enregistrer
					</Button>
					{survey.status === 'DRAFT' ? (
						<Button
							disabled={saving}
							aria-disabled={blocked || undefined}
							aria-describedby={blocked ? SAVE_REASON_ID : undefined}
							onClick={onOpen}
						>
							<DoorOpen aria-hidden />
							Ouvrir aux réponses
						</Button>
					) : null}
					{survey.status === 'OPENED' ? (
						<Button variant="outline" disabled={saving} onClick={onClose}>
							<DoorClosed aria-hidden />
							Fermer
						</Button>
					) : null}
				</div>
			) : null}
			<div className="grid gap-1">
				<Button
					variant="ghost"
					size="sm"
					className="justify-start"
					onClick={onResponses}
				>
					<Inbox aria-hidden />
					Réponses
				</Button>
				{onCopyLink ? (
					<Button
						variant="ghost"
						size="sm"
						className="justify-start"
						onClick={onCopyLink}
					>
						<Copy aria-hidden />
						Copier le lien public
					</Button>
				) : null}
				{canManage ? (
					<>
						<Button
							variant="ghost"
							size="sm"
							className="justify-start"
							disabled={duplicating}
							onClick={onDuplicate}
						>
							<CopyPlus aria-hidden />
							Dupliquer
						</Button>
						<Button
							variant="ghost"
							size="sm"
							className="justify-start text-destructive hover:text-destructive"
							onClick={onDelete}
						>
							<Trash2 aria-hidden />
							Supprimer
						</Button>
					</>
				) : null}
			</div>
		</div>
	)
}

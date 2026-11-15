import {
	Copy,
	CopyPlus,
	DoorClosed,
	DoorOpen,
	Eye,
	Inbox,
	RotateCcw,
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
	onReopen,
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
	onReopen: () => void
	onDuplicate: () => void
	onDelete: () => void
	onCopyLink: (() => void) | null
}) {
	const blocked = saveBlockedReason !== null
	return (
		<div className="grid divide-y border-b">
			<h1 className="sr-only">{title}</h1>
			<Section title="État">
				<dl className="grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-2 text-sm">
					<dt className="text-muted-foreground">Statut</dt>
					<dd>
						<SurveyStatusBadge status={survey.status} />
					</dd>
					<dt className="text-muted-foreground">Visibilité</dt>
					<dd>
						<VisibilityBadge visibility={survey.visibility} />
					</dd>
					{problems ? (
						<>
							<dt className="text-muted-foreground">Contrôle</dt>
							<dd>{problems}</dd>
						</>
					) : null}
				</dl>
			</Section>
			<Section title="Affichage">
				<label
					htmlFor="survey-preview-switch"
					className="flex items-start justify-between gap-3 text-sm"
				>
					<span className="grid gap-0.5">
						<span className="flex items-center gap-2 font-medium">
							<Eye className="size-4" aria-hidden />
							Aperçu
						</span>
						<span className="text-muted-foreground text-xs">
							Voir le questionnaire comme le répondant le verra.
						</span>
					</span>
					<Switch
						id="survey-preview-switch"
						checked={previewing}
						onCheckedChange={onPreviewing}
					/>
				</label>
			</Section>
			{canManage ? (
				<Section title="Publication">
					<div className="grid gap-2">
						<Button
							variant={dirty ? 'default' : 'outline'}
							disabled={!dirty || saving}
							aria-disabled={blocked || undefined}
							aria-describedby={SAVE_REASON_ID}
							onClick={onSave}
						>
							<Save aria-hidden />
							Enregistrer
						</Button>
						<p id={SAVE_REASON_ID} className="text-muted-foreground text-xs">
							{blocked
								? saveBlockedReason
								: dirty
									? 'Des modifications ne sont pas enregistrées.'
									: 'Toutes les modifications sont enregistrées.'}
						</p>
						{survey.status === 'DRAFT' ? (
							<Button
								disabled={saving}
								aria-disabled={blocked || undefined}
								onClick={onOpen}
							>
								<DoorOpen aria-hidden />
								Ouvrir aux réponses
							</Button>
						) : null}
						{survey.status === 'OPENED' ? (
							<Button variant="outline" disabled={saving} onClick={onClose}>
								<DoorClosed aria-hidden />
								Fermer les réponses
							</Button>
						) : null}
						{survey.status === 'CLOSED' ? (
							<Button variant="outline" disabled={saving} onClick={onReopen}>
								<RotateCcw aria-hidden />
								Rouvrir les réponses
							</Button>
						) : null}
					</div>
				</Section>
			) : null}
			<Section title="Autres actions">
				<div className="grid gap-1">
					<Button
						variant="ghost"
						size="sm"
						className="justify-start"
						onClick={onResponses}
					>
						<Inbox aria-hidden />
						Voir les réponses
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
			</Section>
		</div>
	)
}

function Section({ title, children }: { title: string; children: ReactNode }) {
	return (
		<section className="grid gap-3 p-4">
			<h2 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">
				{title}
			</h2>
			{children}
		</section>
	)
}

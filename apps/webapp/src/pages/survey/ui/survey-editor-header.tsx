import {
	ArrowLeft,
	Copy,
	CopyPlus,
	DoorOpen,
	Inbox,
	Lock,
	Save,
	Settings2,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { Survey } from '#/pages/survey/types'
import {
	SurveyStatusBadge,
	VisibilityBadge,
} from '#/pages/survey/ui/survey-badges'

export function SurveyEditorHeader({
	survey,
	title,
	canManage,
	dirty,
	saving,
	canSave,
	onBack,
	onSettings,
	onResponses,
	onSave,
	onOpen,
	onDuplicate,
	onCopyLink,
}: {
	survey: Survey
	title: string
	canManage: boolean
	dirty: boolean
	saving: boolean
	canSave: boolean
	onBack: () => void
	onSettings: () => void
	onResponses: () => void
	onSave: () => void
	onOpen: () => void
	onDuplicate: () => void
	onCopyLink: (() => void) | null
}) {
	const draft = survey.status === 'DRAFT'
	return (
		<header className="flex flex-wrap items-center justify-between gap-3 border-b bg-background px-4 py-2.5">
			<div className="flex min-w-0 items-center gap-2">
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Retour aux questionnaires"
					onClick={onBack}
				>
					<ArrowLeft aria-hidden />
				</Button>
				<h1 className="truncate font-semibold">{title}</h1>
				<SurveyStatusBadge status={survey.status} />
				<VisibilityBadge visibility={survey.visibility} />
				{dirty ? (
					<span className="text-muted-foreground text-xs">
						Modifications non enregistrées
					</span>
				) : null}
			</div>
			<div className="flex flex-wrap items-center gap-2">
				<Button variant="outline" size="sm" onClick={onSettings}>
					<Settings2 aria-hidden />
					Réglages
				</Button>
				<Button variant="outline" size="sm" onClick={onResponses}>
					<Inbox aria-hidden />
					Réponses
				</Button>
				{onCopyLink ? (
					<Button variant="outline" size="sm" onClick={onCopyLink}>
						<Copy aria-hidden />
						Copier le lien public
					</Button>
				) : null}
				{canManage && draft ? (
					<>
						<Button
							variant="outline"
							size="sm"
							disabled={!dirty || saving || !canSave}
							onClick={onSave}
						>
							<Save aria-hidden />
							Enregistrer
						</Button>
						<Button size="sm" disabled={saving || !canSave} onClick={onOpen}>
							<DoorOpen aria-hidden />
							Ouvrir aux réponses
						</Button>
					</>
				) : null}
				{canManage && !draft ? (
					<Button size="sm" onClick={onDuplicate}>
						<CopyPlus aria-hidden />
						Dupliquer pour corriger
					</Button>
				) : null}
			</div>
		</header>
	)
}

export function ReadOnlyBanner({ survey }: { survey: Survey }) {
	if (survey.status === 'DRAFT') return null
	return (
		<output className="flex items-start gap-2 border-b bg-warning-soft px-4 py-2 text-sm text-warning">
			<Lock className="mt-0.5 size-4 shrink-0" aria-hidden />
			<span>
				{survey.status === 'OPENED'
					? 'Ce questionnaire est ouvert : sa définition est figée pour que toutes les réponses portent sur les mêmes questions.'
					: 'Ce questionnaire est fermé : sa définition est figée.'}{' '}
				Pour corriger une question, dupliquez-le en brouillon.
			</span>
		</output>
	)
}

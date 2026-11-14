import {
	ArrowLeft,
	CircleCheck,
	Copy,
	CopyPlus,
	DoorOpen,
	Inbox,
	Lock,
	Save,
	Settings2,
	TriangleAlert,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import type { GraphIssue } from '#/pages/survey/lib/graph-validation'
import type { Survey } from '#/pages/survey/types'
import {
	SurveyStatusBadge,
	VisibilityBadge,
} from '#/pages/survey/ui/survey-badges'

export function VerificationControl({
	issues,
	nodeLabel,
	open,
	onOpenChange,
	onSelect,
}: {
	issues: readonly GraphIssue[]
	nodeLabel: (nodeId: string) => string
	open: boolean
	onOpenChange: (open: boolean) => void
	onSelect: (nodeId: string) => void
}) {
	if (issues.length === 0) {
		return (
			<span
				title="Le parcours est complet : chaque chemin mène à une fin."
				className="inline-flex items-center gap-1.5 rounded-md bg-success-soft px-2.5 py-1 font-medium text-success text-xs"
			>
				<CircleCheck className="size-3.5" aria-hidden />
				Parcours complet
			</span>
		)
	}
	return (
		<Popover open={open} onOpenChange={onOpenChange}>
			<PopoverTrigger asChild>
				<Button
					variant="outline"
					size="sm"
					className="border-destructive/40 text-destructive"
					aria-label={`Vérification : ${issues.length === 1 ? 'un problème' : `${issues.length} problèmes`}`}
				>
					<TriangleAlert aria-hidden />
					Vérification
					<span className="rounded-full bg-destructive px-1.5 text-[11px] text-white tabular-nums">
						{issues.length}
					</span>
				</Button>
			</PopoverTrigger>
			<PopoverContent align="center" className="w-96 p-0">
				<section
					aria-label="Problèmes du questionnaire"
					className="grid gap-2 p-3"
				>
					<h2 className="font-medium text-sm">Vérification</h2>
					<p className="text-muted-foreground text-xs">
						{issues.length === 1
							? 'Un problème empêche l’enregistrement et l’ouverture.'
							: `${issues.length} problèmes empêchent l’enregistrement et l’ouverture.`}
					</p>
					<ul className="grid max-h-80 gap-1 overflow-y-auto">
						{issues.map((item) => {
							const nodeId = item.nodeId
							const content = (
								<>
									<TriangleAlert
										className="mt-0.5 size-3.5 shrink-0 text-destructive"
										aria-hidden
									/>
									<span>
										{nodeId ? (
											<span className="block font-medium">
												{nodeLabel(nodeId)}
											</span>
										) : null}
										<span className="block text-muted-foreground">
											{item.message}
										</span>
									</span>
								</>
							)
							return (
								<li key={`${item.code}:${nodeId ?? 'graph'}`}>
									{nodeId ? (
										<button
											type="button"
											className="flex w-full gap-2 rounded-md px-2 py-1.5 text-left text-xs hover:bg-accent"
											onClick={() => onSelect(nodeId)}
										>
											{content}
										</button>
									) : (
										<div className="flex gap-2 px-2 py-1.5 text-xs">
											{content}
										</div>
									)}
								</li>
							)
						})}
					</ul>
				</section>
			</PopoverContent>
		</Popover>
	)
}

export function SurveyEditorHeader({
	survey,
	title,
	canManage,
	dirty,
	saving,
	duplicating,
	canSave,
	verification,
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
	duplicating: boolean
	canSave: boolean
	verification: ReactNode
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
		<header className="grid min-h-[52px] shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-4 border-border border-b bg-card px-4 py-2">
			<div className="flex min-w-0 items-center gap-2">
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Retour aux questionnaires"
					onClick={onBack}
				>
					<ArrowLeft aria-hidden />
				</Button>
				<nav
					aria-label="Fil d’Ariane"
					className="flex min-w-0 items-center gap-1 text-base"
				>
					<span className="hidden text-muted-foreground md:inline">
						Questionnaires
					</span>
					<span className="hidden text-muted-foreground md:inline" aria-hidden>
						/
					</span>
					<h1 className="truncate font-medium">{title}</h1>
				</nav>
				<SurveyStatusBadge status={survey.status} />
				<VisibilityBadge visibility={survey.visibility} />
				{dirty ? (
					<span className="hidden text-muted-foreground text-xs lg:inline">
						Modifications non enregistrées
					</span>
				) : null}
			</div>
			<div className="flex items-center justify-center">{verification}</div>
			<div className="flex flex-wrap items-center justify-end gap-2">
				<Button variant="ghost" size="sm" onClick={onSettings}>
					<Settings2 aria-hidden />
					Réglages
				</Button>
				<Button variant="ghost" size="sm" onClick={onResponses}>
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
							variant={dirty ? 'default' : 'outline'}
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
					<Button size="sm" disabled={duplicating} onClick={onDuplicate}>
						<CopyPlus aria-hidden />
						Dupliquer pour corriger
					</Button>
				) : null}
			</div>
		</header>
	)
}

export function ReadOnlyBanner({
	survey,
	canManage,
}: {
	survey: Survey
	canManage: boolean
}) {
	if (survey.status === 'DRAFT') {
		if (canManage) return null
		return (
			<output className="flex items-start gap-2 border-b bg-warning-soft px-4 py-2 text-sm text-warning">
				<Lock className="mt-0.5 size-4 shrink-0" aria-hidden />
				<span>
					Lecture seule : vous n’avez pas la permission de modifier ce
					questionnaire.
				</span>
			</output>
		)
	}
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

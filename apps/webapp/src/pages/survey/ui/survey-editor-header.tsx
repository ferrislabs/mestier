import {
	ArrowLeft,
	CircleCheck,
	Copy,
	CopyPlus,
	DoorClosed,
	DoorOpen,
	Eye,
	Inbox,
	MoreHorizontal,
	Save,
	Settings2,
	TriangleAlert,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import type { Survey } from '#/pages/survey/types'
import {
	SurveyStatusBadge,
	VisibilityBadge,
} from '#/pages/survey/ui/survey-badges'

export const SAVE_REASON_ID = 'survey-save-blocked-reason'

export function ProblemsSummary({
	count,
	onFocusFirst,
}: {
	count: number
	onFocusFirst: () => void
}) {
	if (count === 0) {
		return (
			<span className="inline-flex items-center gap-1.5 rounded-md bg-success-soft px-2.5 py-1 font-medium text-success text-xs">
				<CircleCheck className="size-3.5" aria-hidden />
				Prêt
			</span>
		)
	}
	return (
		<Button
			type="button"
			variant="outline"
			size="sm"
			className="min-h-11 border-destructive/40 text-destructive sm:min-h-8"
			onClick={onFocusFirst}
		>
			<TriangleAlert aria-hidden />
			{count === 1 ? '1 problème à corriger' : `${count} problèmes à corriger`}
		</Button>
	)
}

export function SurveyEditorHeader({
	survey,
	title,
	titleEditable,
	canManage,
	dirty,
	saving,
	duplicating,
	saveBlockedReason,
	problems,
	onTitle,
	onBack,
	onPreview,
	onSettings,
	onResponses,
	onSave,
	onOpen,
	onClose,
	onDuplicate,
	onCopyLink,
}: {
	survey: Survey
	title: string
	titleEditable: boolean
	canManage: boolean
	dirty: boolean
	saving: boolean
	duplicating: boolean
	saveBlockedReason: string | null
	problems: ReactNode
	onTitle: (title: string) => void
	onBack: () => void
	onPreview: () => void
	onSettings: () => void
	onResponses: () => void
	onSave: () => void
	onOpen: () => void
	onClose: () => void
	onDuplicate: () => void
	onCopyLink: (() => void) | null
}) {
	const draft = survey.status === 'DRAFT'
	const blocked = saveBlockedReason !== null
	return (
		<header className="grid shrink-0 gap-2 border-border border-b bg-card px-3 py-2 sm:px-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
			<div className="flex min-w-0 items-center gap-2">
				<Button
					variant="ghost"
					size="icon"
					className="size-11 shrink-0 sm:size-8"
					aria-label="Retour aux questionnaires"
					onClick={onBack}
				>
					<ArrowLeft aria-hidden />
				</Button>
				<h1 className="sr-only">{title}</h1>
				{titleEditable ? (
					<input
						aria-label="Titre du questionnaire"
						value={title}
						placeholder="Questionnaire sans titre"
						className="min-h-11 min-w-0 flex-1 truncate rounded-md bg-transparent px-2 font-medium text-base outline-none hover:bg-accent/50 focus-visible:ring-[3px] focus-visible:ring-ring/50 sm:min-h-8"
						onChange={(event) => onTitle(event.target.value)}
					/>
				) : (
					<span aria-hidden className="min-w-0 truncate font-medium">
						{title}
					</span>
				)}
				<SurveyStatusBadge status={survey.status} />
				<span className="hidden sm:inline-flex">
					<VisibilityBadge visibility={survey.visibility} />
				</span>
				{dirty ? (
					<span className="hidden text-muted-foreground text-xs xl:inline">
						Modifications non enregistrées
					</span>
				) : null}
			</div>
			<div className="flex flex-wrap items-center gap-1.5 lg:justify-end">
				{problems}
				{canManage && blocked ? (
					<span
						id={SAVE_REASON_ID}
						className="max-w-64 text-muted-foreground text-xs"
					>
						{saveBlockedReason}
					</span>
				) : null}
				<Button
					variant="ghost"
					size="sm"
					className="min-h-11 sm:min-h-8"
					onClick={onPreview}
				>
					<Eye aria-hidden />
					Aperçu
				</Button>
				<Button
					variant="ghost"
					size="sm"
					className="min-h-11 sm:min-h-8"
					onClick={onSettings}
				>
					<Settings2 aria-hidden />
					Réglages
				</Button>
				<Button
					variant="ghost"
					size="sm"
					className="min-h-11 sm:min-h-8"
					onClick={onResponses}
				>
					<Inbox aria-hidden />
					Réponses
				</Button>
				{canManage ? (
					<>
						<Button
							variant={dirty ? 'default' : 'outline'}
							size="sm"
							className="min-h-11 aria-disabled:opacity-50 sm:min-h-8"
							disabled={!dirty || saving}
							aria-disabled={blocked || undefined}
							aria-describedby={blocked ? SAVE_REASON_ID : undefined}
							onClick={onSave}
						>
							<Save aria-hidden />
							Enregistrer
						</Button>
						{draft ? (
							<Button
								size="sm"
								className="min-h-11 aria-disabled:opacity-50 sm:min-h-8"
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
							<Button
								variant="outline"
								size="sm"
								className="min-h-11 sm:min-h-8"
								disabled={saving}
								onClick={onClose}
							>
								<DoorClosed aria-hidden />
								Fermer
							</Button>
						) : null}
					</>
				) : null}
				{canManage || onCopyLink ? (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								variant="ghost"
								size="icon"
								className="size-11 sm:size-8"
								aria-label="Plus d’actions"
							>
								<MoreHorizontal aria-hidden />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							{onCopyLink ? (
								<DropdownMenuItem onSelect={onCopyLink}>
									<Copy aria-hidden />
									Copier le lien public
								</DropdownMenuItem>
							) : null}
							{canManage ? (
								<DropdownMenuItem disabled={duplicating} onSelect={onDuplicate}>
									<CopyPlus aria-hidden />
									Dupliquer
								</DropdownMenuItem>
							) : null}
						</DropdownMenuContent>
					</DropdownMenu>
				) : null}
			</div>
		</header>
	)
}

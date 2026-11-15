import {
	ArrowDown,
	ArrowUp,
	ChevronDown,
	CopyPlus,
	GitBranch,
	GripVertical,
	Trash2,
	TriangleAlert,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import { MAX_PHOTOS_LIMIT } from '#/pages/survey/lib/graph-validation'
import {
	isChoiceKind,
	isIdentifierKind,
	questionKindEntry,
} from '#/pages/survey/lib/question-kinds'
import type {
	FormQuestion,
	RuleCondition,
	RuleJoin,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import { QuestionKindMenu } from '#/pages/survey/ui/question-kind-menu'
import { QuestionLogic } from '#/pages/survey/ui/question-logic'
import { QuestionOptions } from '#/pages/survey/ui/question-options'

export type QuestionAction =
	| { type: 'label'; value: string }
	| { type: 'kind'; kind: QuestionKind }
	| { type: 'required'; value: boolean }
	| { type: 'add-option' }
	| { type: 'rename-option'; position: number; value: string }
	| { type: 'remove-option'; position: number }
	| { type: 'max-photos'; value: number | null }
	| { type: 'identifier'; value: boolean }
	| { type: 'move'; delta: number }
	| { type: 'duplicate' }
	| { type: 'delete' }
	| { type: 'toggle-logic' }
	| { type: 'add-condition' }
	| {
			type: 'update-condition'
			position: number
			patch: Partial<RuleCondition>
	  }
	| { type: 'remove-condition'; position: number }
	| { type: 'join'; join: RuleJoin }
	| { type: 'clear-rule' }

export function questionCardId(id: string): string {
	return `question-card-${id}`
}

export function questionLabelId(id: string): string {
	return `question-${id}-label`
}

export function questionHandleId(id: string): string {
	return `question-${id}-handle`
}

export const DEFAULT_MAX_PHOTOS = 5

export function QuestionCard({
	question,
	number,
	total,
	problems,
	highlighted,
	readOnly,
	sources,
	positions,
	isSource,
	identifierOwner,
	logicOpen,
	onAction,
}: {
	question: FormQuestion
	number: number
	total: number
	problems: readonly string[]
	highlighted: boolean
	readOnly: boolean
	sources: readonly FormQuestion[]
	positions: ReadonlyMap<string, number>
	isSource: boolean
	identifierOwner: FormQuestion | undefined
	logicOpen: boolean
	onAction: (action: QuestionAction) => void
}) {
	const entry = questionKindEntry(question.kind)
	const id = question.id
	const labelId = questionLabelId(id)
	const problemsId = `question-${id}-problems`
	const logicId = `question-${id}-logic`
	const requiredId = `question-${id}-required`
	const identifierId = `question-${id}-identifier`
	const photosId = `question-${id}-photos`
	const takenByOther =
		identifierOwner !== undefined && identifierOwner.id !== id
	const ruleCount = question.rule?.conditions.length ?? 0
	return (
		<article
			id={questionCardId(id)}
			aria-labelledby={`question-${id}-number ${labelId}`}
			className={cn(
				'group/card relative grid gap-4 rounded-xl border bg-card p-4 shadow-xs transition-colors focus-within:border-ring sm:p-5',
				problems.length > 0 && 'border-destructive/50',
				highlighted && 'ring-2 ring-destructive/40',
			)}
		>
			<div className="flex items-start gap-2">
				{readOnly ? null : (
					<div className="flex shrink-0 flex-col items-center gap-0.5">
						<Button
							id={questionHandleId(id)}
							type="button"
							variant="ghost"
							size="icon"
							className="size-11 cursor-grab sm:size-8"
							aria-label={`Réordonner la question ${number} : flèches haut et bas`}
							aria-keyshortcuts="ArrowUp ArrowDown"
							onKeyDown={(event) => {
								if (event.key === 'ArrowUp') {
									event.preventDefault()
									onAction({ type: 'move', delta: -1 })
								}
								if (event.key === 'ArrowDown') {
									event.preventDefault()
									onAction({ type: 'move', delta: 1 })
								}
							}}
						>
							<GripVertical aria-hidden />
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="icon"
							className="size-11 sm:size-7"
							aria-label={`Monter la question ${number}`}
							disabled={number === 1}
							onClick={() => onAction({ type: 'move', delta: -1 })}
						>
							<ArrowUp aria-hidden />
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="icon"
							className="size-11 sm:size-7"
							aria-label={`Descendre la question ${number}`}
							disabled={number === total}
							onClick={() => onAction({ type: 'move', delta: 1 })}
						>
							<ArrowDown aria-hidden />
						</Button>
					</div>
				)}
				<div className="grid min-w-0 flex-1 gap-3">
					<div className="flex flex-wrap items-center gap-2">
						<span
							id={`question-${id}-number`}
							className="font-semibold text-muted-foreground text-sm tabular-nums"
						>
							Question {number}
						</span>
						{question.rule ? (
							<span className="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-muted-foreground text-xs">
								<GitBranch className="size-3" aria-hidden />
								Sous condition
							</span>
						) : null}
						<div className="ml-auto">
							{readOnly ? (
								<span className="inline-flex items-center gap-2 text-sm">
									<KindIconTile icon={entry.icon} hue={entry.hue} size="sm" />
									{entry.title}
								</span>
							) : (
								<QuestionKindMenu
									label="Type de question"
									current={question.kind}
									align="end"
									onSelect={(kind) => onAction({ type: 'kind', kind })}
								>
									<Button
										type="button"
										variant="outline"
										size="sm"
										className="min-h-11 sm:min-h-8"
										aria-label={`Type de la question ${number} : ${entry.title}`}
									>
										<KindIconTile icon={entry.icon} hue={entry.hue} size="sm" />
										{entry.title}
										<ChevronDown aria-hidden />
									</Button>
								</QuestionKindMenu>
							)}
						</div>
					</div>
					<label htmlFor={labelId} className="sr-only">
						Libellé de la question {number}
					</label>
					<Textarea
						id={labelId}
						value={question.label}
						rows={1}
						placeholder="Votre question"
						disabled={readOnly}
						aria-invalid={problems.length > 0 || undefined}
						aria-describedby={problems.length > 0 ? problemsId : undefined}
						className="min-h-11 resize-none border-0 border-b bg-transparent px-0 text-lg font-medium shadow-none focus-visible:ring-0 md:text-lg dark:bg-transparent"
						onChange={(event) =>
							onAction({ type: 'label', value: event.target.value })
						}
					/>
				</div>
			</div>
			{isChoiceKind(question.kind) ? (
				<QuestionOptions
					questionId={id}
					options={question.options}
					multiple={question.kind === 'MULTIPLE_CHOICE'}
					readOnly={readOnly}
					onAdd={() => onAction({ type: 'add-option' })}
					onRename={(position, value) =>
						onAction({ type: 'rename-option', position, value })
					}
					onRemove={(position) => onAction({ type: 'remove-option', position })}
				/>
			) : null}
			{question.kind === 'PHOTO' ? (
				<div className="grid max-w-56 gap-1.5">
					<label htmlFor={photosId} className="font-medium text-sm">
						Photos maximum
					</label>
					<Input
						id={photosId}
						type="number"
						min={1}
						max={MAX_PHOTOS_LIMIT}
						inputMode="numeric"
						placeholder={String(DEFAULT_MAX_PHOTOS)}
						value={question.maxPhotos ?? ''}
						disabled={readOnly}
						className="min-h-11 sm:min-h-9"
						onChange={(event) => {
							const raw = event.target.value
							onAction({
								type: 'max-photos',
								value: raw === '' ? null : Number(raw),
							})
						}}
					/>
					<p className="text-muted-foreground text-xs">
						De 1 à {MAX_PHOTOS_LIMIT}, {DEFAULT_MAX_PHOTOS} si vide.
					</p>
				</div>
			) : null}
			{isIdentifierKind(question.kind) ? (
				<div className="flex items-start justify-between gap-3">
					<label htmlFor={identifierId} className="text-sm">
						<span className="block font-medium">Identifiant du répondant</span>
						<span className="block text-muted-foreground text-xs">
							{takenByOther
								? `« ${identifierOwner.label.trim() || 'Question sans libellé'} » sert déjà d’identifiant : une seule question peut l’être.`
								: 'Sert à reconnaître le répondant et à éviter qu’il réponde deux fois.'}
						</span>
					</label>
					<Switch
						id={identifierId}
						checked={question.identifier}
						disabled={readOnly || (takenByOther && !question.identifier)}
						onCheckedChange={(value) => onAction({ type: 'identifier', value })}
					/>
				</div>
			) : null}
			{problems.length > 0 ? (
				<ul id={problemsId} className="grid gap-1">
					{problems.map((message) => (
						<li
							key={message}
							className="flex items-start gap-1.5 text-destructive text-xs"
						>
							<TriangleAlert className="mt-0.5 size-3.5 shrink-0" aria-hidden />
							{message}
						</li>
					))}
				</ul>
			) : null}
			{logicOpen ? (
				<section
					id={logicId}
					aria-label={`Logique de la question ${number}`}
					className="rounded-lg border border-dashed p-3"
				>
					<QuestionLogic
						questionId={id}
						rule={question.rule}
						sources={sources}
						positions={positions}
						isSource={isSource}
						readOnly={readOnly}
						onAddCondition={() => onAction({ type: 'add-condition' })}
						onUpdateCondition={(position, patch) =>
							onAction({ type: 'update-condition', position, patch })
						}
						onRemoveCondition={(position) =>
							onAction({ type: 'remove-condition', position })
						}
						onJoin={(join) => onAction({ type: 'join', join })}
						onClear={() => onAction({ type: 'clear-rule' })}
					/>
				</section>
			) : null}
			<div className="flex flex-wrap items-center gap-1 border-t pt-3">
				<Button
					type="button"
					variant="ghost"
					size="sm"
					className="min-h-11 sm:min-h-8"
					aria-expanded={logicOpen}
					aria-controls={logicOpen ? logicId : undefined}
					onClick={() => onAction({ type: 'toggle-logic' })}
				>
					<GitBranch aria-hidden />
					Logique
					{ruleCount > 0 ? (
						<span className="rounded-full bg-muted px-1.5 text-[11px] tabular-nums">
							{ruleCount}
						</span>
					) : null}
					<ChevronDown
						aria-hidden
						className={cn('transition-transform', logicOpen && 'rotate-180')}
					/>
				</Button>
				<div className="ml-auto flex flex-wrap items-center gap-1">
					<div className="flex items-center gap-2 pr-2">
						<label htmlFor={requiredId} className="text-sm">
							Obligatoire
						</label>
						<Switch
							id={requiredId}
							checked={question.required}
							disabled={readOnly}
							onCheckedChange={(value) => onAction({ type: 'required', value })}
						/>
					</div>
					{readOnly ? null : (
						<>
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className="size-11 sm:size-8"
								aria-label={`Dupliquer la question ${number}`}
								onClick={() => onAction({ type: 'duplicate' })}
							>
								<CopyPlus aria-hidden />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className="size-11 text-destructive sm:size-8"
								aria-label={`Supprimer la question ${number}`}
								onClick={() => onAction({ type: 'delete' })}
							>
								<Trash2 aria-hidden />
							</Button>
						</>
					)}
				</div>
			</div>
		</article>
	)
}

import { Plus, Trash2, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type {
	DisplayRule,
	FormQuestion,
	RuleCondition,
	RuleJoin,
} from '#/pages/survey/lib/survey-form'
import { NATIVE_SELECT } from '#/pages/survey/ui/native-select'

export const SOURCE_HINT =
	'Seules les questions toujours posées peuvent servir de condition.'

function sourceLabel(item: FormQuestion, position: number): string {
	const label = item.label.trim()
	return `${position}. ${label === '' ? 'Question sans libellé' : label}`
}

function optionEntries(
	source: FormQuestion | undefined,
): { key: string; option: string }[] {
	if (!source) return []
	return source.options.flatMap((option, at) =>
		option.trim() === '' ? [] : [{ key: `${source.id}-${at}`, option }],
	)
}

export type LogicSubject = 'question' | 'block'

const COPY = {
	question: {
		always: 'Cette question est toujours posée.',
		sourceLocked:
			'Elle sert de condition à d’autres questions : elle doit rester toujours posée.',
		noSource:
			'Ajoutez avant elle une question à choix toujours posée pour l’afficher sous condition.',
		lead: 'Afficher cette question seulement si',
		clear: 'Toujours poser cette question',
	},
	block: {
		always: 'Ce bloc est toujours affiché.',
		sourceLocked:
			'Une de ses questions sert de condition à d’autres questions : le bloc doit rester toujours affiché.',
		noSource:
			'Ajoutez avant le bloc une question à choix toujours posée pour l’afficher sous condition.',
		lead: 'Afficher ce bloc seulement si',
		clear: 'Toujours afficher ce bloc',
	},
} as const

export function QuestionLogic({
	questionId,
	subject = 'question',
	rule,
	sources,
	positions,
	isSource,
	readOnly,
	onAddCondition,
	onUpdateCondition,
	onRemoveCondition,
	onJoin,
	onClear,
}: {
	questionId: string
	subject?: LogicSubject
	rule: DisplayRule | null
	sources: readonly FormQuestion[]
	positions: ReadonlyMap<string, number>
	isSource: boolean
	readOnly: boolean
	onAddCondition: () => void
	onUpdateCondition: (position: number, patch: Partial<RuleCondition>) => void
	onRemoveCondition: (position: number) => void
	onJoin: (join: RuleJoin) => void
	onClear: () => void
}) {
	const copy = COPY[subject]
	if (!rule) {
		return (
			<div className="grid gap-2 text-sm">
				<p className="text-muted-foreground">{copy.always}</p>
				{isSource ? (
					<p className="text-muted-foreground text-xs">{copy.sourceLocked}</p>
				) : sources.length === 0 ? (
					<p className="text-muted-foreground text-xs">
						{copy.noSource} {SOURCE_HINT}
					</p>
				) : readOnly ? null : (
					<div>
						<Button
							type="button"
							variant="outline"
							size="sm"
							className="min-h-11 sm:min-h-8"
							onClick={onAddCondition}
						>
							<Plus aria-hidden />
							Afficher sous condition
						</Button>
					</div>
				)}
			</div>
		)
	}
	const joinId = `question-${questionId}-join`
	const byId = new Map(sources.map((item) => [item.id, item]))
	return (
		<div className="grid gap-3 text-sm">
			<div className="flex flex-wrap items-center gap-2">
				<span>{copy.lead}</span>
				{rule.conditions.length < 2 ? (
					<span>la condition suivante est remplie :</span>
				) : (
					<>
						<label htmlFor={joinId} className="sr-only">
							Conditions à remplir
						</label>
						<select
							id={joinId}
							className={`${NATIVE_SELECT} w-auto min-h-11 sm:min-h-9`}
							value={rule.join}
							disabled={readOnly}
							onChange={(event) => onJoin(event.target.value as RuleJoin)}
						>
							<option value="ALL">toutes</option>
							<option value="ANY">au moins une</option>
						</select>
						<span>les conditions sont remplies :</span>
					</>
				)}
			</div>
			<ol className="grid gap-2">
				{rule.conditions.map((condition, position) => {
					const base = `question-${questionId}-condition-${position}`
					const source = byId.get(condition.question)
					return (
						<li
							key={base}
							className="grid gap-2 rounded-md border bg-muted/30 p-2 sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto_auto] sm:items-center"
						>
							<label htmlFor={`${base}-question`} className="sr-only">
								Question de la condition {position + 1}
							</label>
							<select
								id={`${base}-question`}
								className={`${NATIVE_SELECT} min-h-11 sm:min-h-9`}
								value={source ? condition.question : ''}
								disabled={readOnly}
								onChange={(event) =>
									onUpdateCondition(position, { question: event.target.value })
								}
							>
								<option value="" disabled>
									Choisir une question
								</option>
								{sources.map((item) => (
									<option key={item.id} value={item.id}>
										{sourceLabel(item, positions.get(item.id) ?? 0)}
									</option>
								))}
							</select>
							<label htmlFor={`${base}-option`} className="sr-only">
								Option de la condition {position + 1}
							</label>
							<select
								id={`${base}-option`}
								className={`${NATIVE_SELECT} min-h-11 sm:min-h-9`}
								value={
									source?.options.includes(condition.option)
										? condition.option
										: ''
								}
								disabled={readOnly || !source}
								onChange={(event) =>
									onUpdateCondition(position, { option: event.target.value })
								}
							>
								<option value="" disabled>
									Choisir une option
								</option>
								{optionEntries(source).map((entry) => (
									<option key={entry.key} value={entry.option}>
										l’option « {entry.option} »
									</option>
								))}
							</select>
							<label htmlFor={`${base}-checked`} className="sr-only">
								État de l’option, condition {position + 1}
							</label>
							<select
								id={`${base}-checked`}
								className={`${NATIVE_SELECT} min-h-11 w-auto sm:min-h-9`}
								value={condition.checked ? 'checked' : 'unchecked'}
								disabled={readOnly}
								onChange={(event) =>
									onUpdateCondition(position, {
										checked: event.target.value === 'checked',
									})
								}
							>
								<option value="checked">est cochée</option>
								<option value="unchecked">n’est pas cochée</option>
							</select>
							{readOnly ? null : (
								<Button
									type="button"
									variant="ghost"
									size="icon"
									className="size-11 justify-self-end sm:size-9"
									aria-label={`Retirer la condition ${position + 1}`}
									onClick={() => onRemoveCondition(position)}
								>
									<X aria-hidden />
								</Button>
							)}
						</li>
					)
				})}
			</ol>
			<p className="text-muted-foreground text-xs">{SOURCE_HINT}</p>
			{readOnly ? null : (
				<div className="flex flex-wrap gap-2">
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="min-h-11 sm:min-h-8"
						onClick={onAddCondition}
					>
						<Plus aria-hidden />
						Ajouter une condition
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="min-h-11 sm:min-h-8"
						onClick={onClear}
					>
						<Trash2 aria-hidden />
						{copy.clear}
					</Button>
				</div>
			)}
		</div>
	)
}

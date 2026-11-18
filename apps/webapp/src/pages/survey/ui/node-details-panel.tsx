import { Plus, Trash2, X } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import { MAX_PHOTOS_LIMIT } from '#/pages/survey/lib/graph-validation'
import {
	CONDITION_ENTRY,
	END_ENTRY,
	isChoiceKind,
	isIdentifierKind,
	questionKindEntry,
} from '#/pages/survey/lib/question-kinds'
import { questionLabel } from '#/pages/survey/lib/survey-graph'
import type {
	ConditionNode,
	EndNode,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

export const NATIVE_SELECT =
	'h-9 w-full rounded-md border border-input bg-card px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-60'

function SwitchRow({
	id,
	label,
	hint,
	checked,
	disabled,
	onChange,
}: {
	id: string
	label: string
	hint?: string
	checked: boolean
	disabled: boolean
	onChange: (checked: boolean) => void
}) {
	return (
		<div className="flex items-start justify-between gap-3">
			<label htmlFor={id} className="text-sm">
				<span className="block font-medium">{label}</span>
				{hint ? (
					<span className="block text-muted-foreground text-xs">{hint}</span>
				) : null}
			</label>
			<Switch
				id={id}
				checked={checked}
				disabled={disabled}
				onCheckedChange={onChange}
			/>
		</div>
	)
}

export function OptionsEditor({
	options,
	keys,
	readOnly,
	onChange,
	onAdd,
	onRemove,
}: {
	options: readonly string[]
	keys: readonly string[]
	readOnly: boolean
	onChange: (options: string[]) => void
	onAdd: () => void
	onRemove: (position: number) => void
}) {
	return (
		<div className="grid gap-2">
			<ul className="grid gap-2">
				{options.map((option, position) => (
					<li key={keys[position] ?? option} className="flex gap-2">
						<Input
							aria-label={`Option ${position + 1}`}
							value={option}
							disabled={readOnly}
							onChange={(event) =>
								onChange(
									options.map((current, index) =>
										index === position ? event.target.value : current,
									),
								)
							}
						/>
						{readOnly ? null : (
							<Button
								variant="ghost"
								size="icon"
								aria-label={`Retirer l’option ${position + 1}`}
								onClick={() => onRemove(position)}
							>
								<X aria-hidden />
							</Button>
						)}
					</li>
				))}
			</ul>
			{readOnly ? null : (
				<Button variant="outline" size="sm" className="w-fit" onClick={onAdd}>
					<Plus aria-hidden />
					Ajouter une option
				</Button>
			)}
		</div>
	)
}

function QuestionDetails({
	node,
	optionKeys,
	readOnly,
	otherIdentifier,
	onChange,
	onAddOption,
	onRemoveOption,
}: {
	node: QuestionNode
	optionKeys: readonly string[]
	readOnly: boolean
	otherIdentifier: boolean
	onChange: (node: GraphNode) => void
	onAddOption: () => void
	onRemoveOption: (position: number) => void
}) {
	return (
		<>
			<Field label="Libellé" htmlFor="node-label">
				<Textarea
					id="node-label"
					value={node.label}
					disabled={readOnly}
					placeholder="La question posée au répondant"
					onChange={(event) => onChange({ ...node, label: event.target.value })}
				/>
			</Field>
			<SwitchRow
				id="node-required"
				label="Obligatoire"
				hint="Le répondant ne peut pas terminer sans y répondre, si elle est sur son chemin."
				checked={node.required}
				disabled={readOnly}
				onChange={(required) => onChange({ ...node, required })}
			/>
			{isChoiceKind(node.kind) ? (
				<Field label="Options" htmlFor={null}>
					<OptionsEditor
						options={node.options ?? []}
						keys={optionKeys}
						readOnly={readOnly}
						onChange={(options) => onChange({ ...node, options })}
						onAdd={onAddOption}
						onRemove={onRemoveOption}
					/>
				</Field>
			) : null}
			{node.kind === 'PHOTO' ? (
				<Field label="Nombre maximal de photos" htmlFor="node-max-photos">
					<Input
						id="node-max-photos"
						type="number"
						min={1}
						max={MAX_PHOTOS_LIMIT}
						placeholder="5"
						value={node.max_photos ?? ''}
						disabled={readOnly}
						onChange={(event) => {
							const raw = event.target.value
							onChange({
								...node,
								max_photos: raw === '' ? null : Number.parseInt(raw, 10),
							})
						}}
					/>
					<p className="text-muted-foreground text-xs">
						De 1 à {MAX_PHOTOS_LIMIT}. Vide : 5 photos au plus.
					</p>
				</Field>
			) : null}
			{isIdentifierKind(node.kind) ? (
				<SwitchRow
					id="node-identifier"
					label="Identifiant du répondant"
					hint={
						otherIdentifier && !node.identifier
							? 'Une autre question sert déjà d’identifiant.'
							: 'Sa valeur ne peut servir qu’à une seule réponse complète, sauf si les réponses multiples sont autorisées.'
					}
					checked={Boolean(node.identifier)}
					disabled={readOnly || (otherIdentifier && !node.identifier)}
					onChange={(identifier) => onChange({ ...node, identifier })}
				/>
			) : null}
		</>
	)
}

function unlistedLabel(question: GraphNode | null): string {
	if (!question) return 'Question supprimée'
	if (question.type !== 'QUESTION') return 'Nœud qui n’est pas une question'
	return `${questionLabel(question)} (hors des questions précédentes)`
}

function ConditionDetails({
	node,
	choiceQuestions,
	conditionQuestion,
	sentence,
	readOnly,
	onChange,
}: {
	node: ConditionNode
	choiceQuestions: readonly QuestionNode[]
	conditionQuestion: GraphNode | null
	sentence: string
	readOnly: boolean
	onChange: (node: GraphNode) => void
}) {
	const source = choiceQuestions.find(
		(question) => question.id === node.question,
	)
	const unlisted = node.question !== '' && !source
	return (
		<>
			<p className="rounded-md bg-muted px-3 py-2 text-sm">{sentence}</p>
			<Field label="Question testée" htmlFor="condition-question">
				<select
					id="condition-question"
					className={NATIVE_SELECT}
					value={node.question}
					disabled={readOnly}
					aria-invalid={unlisted || undefined}
					onChange={(event) =>
						onChange({ ...node, question: event.target.value, option: '' })
					}
				>
					<option value="">Choisir une question à choix…</option>
					{unlisted ? (
						<option value={node.question} disabled>
							{unlistedLabel(conditionQuestion)}
						</option>
					) : null}
					{choiceQuestions.map((question) => (
						<option key={question.id} value={question.id}>
							{questionLabel(question)}
						</option>
					))}
				</select>
				{choiceQuestions.length === 0 ? (
					<p className="text-muted-foreground text-xs">
						Ajoutez d’abord une question à choix unique ou multiple avant la
						condition.
					</p>
				) : null}
			</Field>
			<Field label="Option" htmlFor="condition-option">
				<select
					id="condition-option"
					className={NATIVE_SELECT}
					value={node.option}
					disabled={readOnly || !source}
					onChange={(event) =>
						onChange({ ...node, option: event.target.value })
					}
				>
					<option value="">Choisir une option…</option>
					{(source?.options ?? []).map((option) => (
						<option key={option} value={option}>
							{option}
						</option>
					))}
				</select>
			</Field>
			<Field label="État de l’option" htmlFor="condition-checked">
				<select
					id="condition-checked"
					className={NATIVE_SELECT}
					value={node.checked ? 'checked' : 'unchecked'}
					disabled={readOnly}
					onChange={(event) =>
						onChange({ ...node, checked: event.target.value === 'checked' })
					}
				>
					<option value="checked">est cochée</option>
					<option value="unchecked">n’est pas cochée</option>
				</select>
			</Field>
			<p className="text-muted-foreground text-xs">
				La sortie Oui est suivie quand la condition est vraie, la sortie Non
				sinon.
			</p>
		</>
	)
}

function EndDetails({
	node,
	readOnly,
	onChange,
}: {
	node: EndNode
	readOnly: boolean
	onChange: (node: GraphNode) => void
}) {
	return (
		<Field label="Message de fin" htmlFor="end-message">
			<Textarea
				id="end-message"
				value={node.message ?? ''}
				disabled={readOnly}
				placeholder="Merci pour vos réponses !"
				onChange={(event) => onChange({ ...node, message: event.target.value })}
			/>
			<p className="text-muted-foreground text-xs">
				Affiché au répondant quand son chemin se termine ici.
			</p>
		</Field>
	)
}

function heading(node: GraphNode): { title: string; icon: ReactNode } {
	if (node.type === 'QUESTION') {
		const entry = questionKindEntry(node.kind)
		const Icon = entry.icon
		return { title: entry.title, icon: <Icon className="size-4" aria-hidden /> }
	}
	const entry = node.type === 'CONDITION' ? CONDITION_ENTRY : END_ENTRY
	const Icon = entry.icon
	return { title: entry.title, icon: <Icon className="size-4" aria-hidden /> }
}

export function NodeDetailsPanel({
	node,
	issues,
	choiceQuestions,
	conditionQuestion,
	sentence,
	optionKeys,
	otherIdentifier,
	isStart,
	readOnly,
	onChange,
	onAddOption,
	onRemoveOption,
	onMakeStart,
	onDelete,
	onClose,
}: {
	node: GraphNode
	issues: readonly string[]
	choiceQuestions: readonly QuestionNode[]
	conditionQuestion: GraphNode | null
	sentence: string
	optionKeys: readonly string[]
	otherIdentifier: boolean
	isStart: boolean
	readOnly: boolean
	onChange: (node: GraphNode) => void
	onAddOption: () => void
	onRemoveOption: (position: number) => void
	onMakeStart: () => void
	onDelete: () => void
	onClose: () => void
}) {
	const { title, icon } = heading(node)
	return (
		<section aria-label="Détails du nœud" className="flex min-h-0 flex-col">
			<header className="flex items-center justify-between gap-2 border-b px-4 py-3">
				<h2 className="flex items-center gap-2 font-medium">
					{icon}
					{title}
					{isStart ? (
						<span className="rounded bg-brand-soft px-1.5 py-0.5 text-[11px] text-primary">
							Départ
						</span>
					) : null}
				</h2>
				<Button
					variant="ghost"
					size="icon-sm"
					aria-label="Fermer les détails"
					onClick={onClose}
				>
					<X aria-hidden />
				</Button>
			</header>
			<div className="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto p-4">
				{issues.length > 0 ? (
					<ul
						aria-label="Problèmes de ce nœud"
						className="grid gap-1 rounded-md border border-destructive/30 bg-destructive-soft px-3 py-2 text-destructive text-xs"
					>
						{issues.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				) : null}
				{node.type === 'QUESTION' ? (
					<QuestionDetails
						node={node}
						optionKeys={optionKeys}
						readOnly={readOnly}
						otherIdentifier={otherIdentifier}
						onChange={onChange}
						onAddOption={onAddOption}
						onRemoveOption={onRemoveOption}
					/>
				) : node.type === 'CONDITION' ? (
					<ConditionDetails
						node={node}
						choiceQuestions={choiceQuestions}
						conditionQuestion={conditionQuestion}
						sentence={sentence}
						readOnly={readOnly}
						onChange={onChange}
					/>
				) : (
					<EndDetails node={node} readOnly={readOnly} onChange={onChange} />
				)}
			</div>
			{readOnly ? null : (
				<footer className="flex flex-wrap justify-between gap-2 border-t px-4 py-3">
					{node.type === 'QUESTION' && !isStart ? (
						<Button variant="outline" size="sm" onClick={onMakeStart}>
							Définir comme départ
						</Button>
					) : (
						<span />
					)}
					<Button variant="ghost" size="sm" onClick={onDelete}>
						<Trash2 aria-hidden />
						Supprimer le nœud
					</Button>
				</footer>
			)}
		</section>
	)
}

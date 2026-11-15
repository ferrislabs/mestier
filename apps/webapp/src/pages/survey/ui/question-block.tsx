import type {
	DraggableAttributes,
	DraggableSyntheticListeners,
} from '@dnd-kit/core'
import {
	ArrowDown,
	ArrowUp,
	ChevronDown,
	GitBranch,
	GripVertical,
	Layers,
	Plus,
} from 'lucide-react'
import type { CSSProperties, ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type {
	FormQuestion,
	RuleCondition,
	RuleJoin,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'
import { QuestionKindMenu } from '#/pages/survey/ui/question-kind-menu'
import { QuestionLogic } from '#/pages/survey/ui/question-logic'

export function blockHandleId(block: string): string {
	return `block-${block}-handle`
}

export interface BlockDrag {
	attributes: DraggableAttributes
	listeners: DraggableSyntheticListeners
	dragging: boolean
}

export type BlockRuleAction =
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

export function QuestionBlock({
	block,
	first,
	from,
	to,
	canMoveUp,
	canMoveDown,
	readOnly,
	logicOpen,
	sources,
	positions,
	isSource,
	drag,
	itemRef,
	style,
	inserter,
	children,
	onMove,
	onAddQuestion,
	onRuleAction,
}: {
	block: string
	first: FormQuestion
	from: number
	to: number
	canMoveUp: boolean
	canMoveDown: boolean
	readOnly: boolean
	logicOpen: boolean
	sources: readonly FormQuestion[]
	positions: ReadonlyMap<string, number>
	isSource: boolean
	drag?: BlockDrag
	itemRef?: (node: HTMLLIElement | null) => void
	style?: CSSProperties
	inserter: ReactNode
	children: ReactNode
	onMove: (delta: number) => void
	onAddQuestion: (kind: QuestionKind) => void
	onRuleAction: (action: BlockRuleAction) => void
}) {
	const size = to - from + 1
	const titleId = `block-${block}-title`
	const logicId = `block-${block}-logic`
	const range = `questions ${from} à ${to}`
	const ruleCount = first.rule?.conditions.length ?? 0
	return (
		<li
			ref={itemRef}
			style={style}
			className={drag?.dragging ? 'relative z-10 grid gap-1' : 'grid gap-1'}
		>
			{inserter}
			<section
				aria-labelledby={titleId}
				className="grid gap-2 rounded-xl border bg-brand-soft/40 p-2 sm:p-3"
			>
				<div className="flex flex-wrap items-center gap-2">
					{readOnly ? null : (
						<div className="flex items-center gap-0.5">
							<Button
								{...drag?.attributes}
								id={blockHandleId(block)}
								type="button"
								variant="ghost"
								size="icon"
								className="size-11 touch-none sm:size-8"
								aria-label={`Réordonner le bloc des ${range} : flèches haut et bas, ou Espace pour le saisir`}
								aria-keyshortcuts="ArrowUp ArrowDown Space"
								onPointerDown={(event) =>
									drag?.listeners?.onPointerDown?.(event)
								}
								onKeyDown={(event) => {
									if (!drag?.dragging && event.key === 'ArrowUp') {
										event.preventDefault()
										onMove(-1)
										return
									}
									if (!drag?.dragging && event.key === 'ArrowDown') {
										event.preventDefault()
										onMove(1)
										return
									}
									drag?.listeners?.onKeyDown?.(event)
								}}
							>
								<GripVertical aria-hidden />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className="size-11 sm:size-8"
								aria-label={`Monter le bloc des ${range}`}
								disabled={!canMoveUp}
								onClick={() => onMove(-1)}
							>
								<ArrowUp aria-hidden />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon"
								className="size-11 sm:size-8"
								aria-label={`Descendre le bloc des ${range}`}
								disabled={!canMoveDown}
								onClick={() => onMove(1)}
							>
								<ArrowDown aria-hidden />
							</Button>
						</div>
					)}
					<h3
						id={titleId}
						className="flex items-center gap-2 font-semibold text-sm"
					>
						<Layers className="size-4 text-primary" aria-hidden />
						Bloc de {size} questions
					</h3>
					<span className="text-muted-foreground text-xs">
						Affichées ensemble, {range}
					</span>
					{first.rule ? (
						<span className="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-muted-foreground text-xs">
							<GitBranch className="size-3" aria-hidden />
							Sous condition
						</span>
					) : null}
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="ml-auto min-h-11 sm:min-h-8"
						aria-expanded={logicOpen}
						aria-controls={logicOpen ? logicId : undefined}
						aria-label={`Logique du bloc des ${range}`}
						onClick={() => onRuleAction({ type: 'toggle-logic' })}
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
				</div>
				{logicOpen ? (
					<section
						id={logicId}
						aria-label="Afficher ce bloc seulement si…"
						className="rounded-lg border border-dashed bg-card p-3"
					>
						<QuestionLogic
							questionId={first.id}
							subject="block"
							rule={first.rule}
							sources={sources}
							positions={positions}
							isSource={isSource}
							readOnly={readOnly}
							onAddCondition={() => onRuleAction({ type: 'add-condition' })}
							onUpdateCondition={(position, patch) =>
								onRuleAction({ type: 'update-condition', position, patch })
							}
							onRemoveCondition={(position) =>
								onRuleAction({ type: 'remove-condition', position })
							}
							onJoin={(join) => onRuleAction({ type: 'join', join })}
							onClear={() => onRuleAction({ type: 'clear-rule' })}
						/>
					</section>
				) : null}
				<ol
					aria-label={`Questions du bloc des ${range}`}
					className="grid gap-1"
				>
					{children}
				</ol>
				{readOnly ? null : (
					<div className="flex justify-center pt-1">
						<QuestionKindMenu
							label="Ajouter une question au bloc"
							current={null}
							align="center"
							keepFocus
							onSelect={onAddQuestion}
						>
							<Button
								type="button"
								variant="outline"
								size="sm"
								className="min-h-11 sm:min-h-8"
							>
								<Plus aria-hidden />
								Ajouter une question au bloc
							</Button>
						</QuestionKindMenu>
					</div>
				)}
			</section>
		</li>
	)
}

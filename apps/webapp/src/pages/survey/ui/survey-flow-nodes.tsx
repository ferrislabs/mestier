import { Handle, type NodeProps, Position } from '@xyflow/react'
import { Asterisk, Fingerprint, Play, TriangleAlert } from 'lucide-react'
import type { CSSProperties, ReactNode } from 'react'
import { cn } from '#/lib/utils'
import {
	CONDITION_ENTRY,
	END_ENTRY,
	questionKindEntry,
} from '#/pages/survey/lib/question-kinds'
import {
	BRANCH_LABELS,
	type SurveyFlowNode,
} from '#/pages/survey/lib/survey-flow'
import { START_HANDLE } from '#/pages/survey/lib/survey-graph'
import '#/pages/automation/ui/automation.css'

function IssueBadge({ issues }: { issues: readonly string[] }) {
	if (issues.length === 0) return null
	return (
		<span
			role="img"
			aria-label={`À corriger : ${issues.join(' ')}`}
			title={issues.join('\n')}
			className="absolute -top-2 -right-2 flex size-5 items-center justify-center rounded-full border-2 border-background bg-destructive text-white"
		>
			<TriangleAlert className="size-3" strokeWidth={2.75} />
		</span>
	)
}

function Card({
	hue,
	selected,
	highlighted,
	issues,
	label,
	children,
	target = true,
	outputs,
}: {
	hue: number
	selected: boolean
	highlighted: boolean
	issues: readonly string[]
	label: string
	children: ReactNode
	target?: boolean
	outputs: readonly { id: string; label?: string }[]
}) {
	return (
		<section
			aria-label={label}
			data-highlighted={highlighted ? 'true' : undefined}
			className={cn(
				'node-tone relative w-[240px] rounded-lg border bg-card px-3 py-2.5 text-left shadow-sm',
				selected && 'ring-2 ring-ring',
				(issues.length > 0 || highlighted) && 'border-2 border-destructive',
				highlighted && 'shadow-[0_0_0_4px] shadow-destructive/25',
			)}
			style={{ '--node-hue': hue } as CSSProperties}
		>
			{target ? (
				<Handle
					type="target"
					position={Position.Left}
					className="automation-handle"
				/>
			) : null}
			{children}
			{outputs.map((output, index) => (
				<Handle
					key={output.id}
					id={output.id}
					type="source"
					position={Position.Right}
					className="automation-handle"
					style={{ top: `${((index + 1) / (outputs.length + 1)) * 100}%` }}
				>
					{output.label ? (
						<span className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 rounded bg-card px-1 font-medium text-[10px] text-muted-foreground">
							{output.label}
						</span>
					) : null}
				</Handle>
			))}
			<IssueBadge issues={issues} />
		</section>
	)
}

function Tile({ hue, children }: { hue: number; children: ReactNode }) {
	return (
		<span
			className="flex size-7 shrink-0 items-center justify-center rounded-md bg-(--tone-bg) text-(--tone-fg)"
			style={{ '--node-hue': hue } as CSSProperties}
		>
			{children}
		</span>
	)
}

export function StartFlowNode() {
	return (
		<section
			aria-label="Départ du questionnaire"
			className="relative flex items-center gap-2 rounded-full border bg-card py-1.5 pr-4 pl-2 font-medium text-sm shadow-sm"
		>
			<span className="flex size-6 items-center justify-center rounded-full bg-primary text-primary-foreground">
				<Play className="size-3" aria-hidden />
			</span>
			Départ
			<Handle
				id={START_HANDLE}
				type="source"
				position={Position.Right}
				className="automation-handle"
			/>
		</section>
	)
}

export function QuestionFlowNode({
	data,
	selected,
}: NodeProps<SurveyFlowNode>) {
	const node = data.node
	if (node?.type !== 'QUESTION') return null
	const entry = questionKindEntry(node.kind)
	const Icon = entry.icon
	const label = node.label.trim() === '' ? 'Question sans libellé' : node.label
	return (
		<Card
			hue={entry.hue}
			selected={selected}
			highlighted={data.highlighted}
			issues={data.issues}
			label={`Question ${entry.title} : ${label}`}
			outputs={[{ id: 'NEXT' }]}
		>
			<div className="flex items-start gap-2.5">
				<Tile hue={entry.hue}>
					<Icon className="size-4" aria-hidden />
				</Tile>
				<div className="min-w-0 flex-1">
					<p className="flex items-center gap-1 text-[11px] text-muted-foreground">
						{entry.title}
						{node.required ? (
							<Asterisk
								className="size-3 text-destructive"
								aria-label="obligatoire"
							/>
						) : null}
						{node.identifier ? (
							<Fingerprint className="size-3" aria-label="identifiant" />
						) : null}
					</p>
					<p
						className={cn(
							'line-clamp-2 font-medium text-sm',
							node.label.trim() === '' && 'text-muted-foreground italic',
						)}
					>
						{label}
					</p>
				</div>
			</div>
		</Card>
	)
}

export function ConditionFlowNode({
	data,
	selected,
}: NodeProps<SurveyFlowNode>) {
	const Icon = CONDITION_ENTRY.icon
	return (
		<Card
			hue={CONDITION_ENTRY.hue}
			selected={selected}
			highlighted={data.highlighted}
			issues={data.issues}
			label={`Condition : ${data.sentence}`}
			outputs={[
				{ id: 'THEN', label: BRANCH_LABELS.THEN },
				{ id: 'ELSE', label: BRANCH_LABELS.ELSE },
			]}
		>
			<div className="flex items-start gap-2.5 pr-6">
				<Tile hue={CONDITION_ENTRY.hue}>
					<Icon className="size-4" aria-hidden />
				</Tile>
				<div className="min-w-0 flex-1">
					<p className="text-[11px] text-muted-foreground">Condition</p>
					<p className="line-clamp-3 text-sm">{data.sentence}</p>
				</div>
			</div>
		</Card>
	)
}

export function EndFlowNode({ data, selected }: NodeProps<SurveyFlowNode>) {
	const node = data.node
	const Icon = END_ENTRY.icon
	const message = node?.type === 'END' ? (node.message ?? '').trim() : ''
	return (
		<Card
			hue={END_ENTRY.hue}
			selected={selected}
			highlighted={data.highlighted}
			issues={data.issues}
			label={`Fin : ${message || 'message par défaut'}`}
			outputs={[]}
		>
			<div className="flex items-start gap-2.5">
				<Tile hue={END_ENTRY.hue}>
					<Icon className="size-4" aria-hidden />
				</Tile>
				<div className="min-w-0 flex-1">
					<p className="text-[11px] text-muted-foreground">Fin</p>
					<p
						className={cn(
							'line-clamp-2 text-sm',
							message === '' && 'text-muted-foreground italic',
						)}
					>
						{message || 'Message de fin par défaut'}
					</p>
				</div>
			</div>
		</Card>
	)
}

export const SURVEY_NODE_TYPES = {
	start: StartFlowNode,
	question: QuestionFlowNode,
	condition: ConditionFlowNode,
	end: EndFlowNode,
}

import type { NodeProps } from '@xyflow/react'
import { Play } from 'lucide-react'
import { AutomationNodeCard } from '#/pages/automation/ui/automation-node-card'
import { useSurveyCanvasActions } from '#/pages/survey/feature/survey-canvas-actions'
import { nodeAppearance, nodeTitle } from '#/pages/survey/lib/node-catalogue'
import {
	BRANCH_LABELS,
	type SurveyFlowNode,
} from '#/pages/survey/lib/survey-flow'
import {
	type ExitHandle,
	exitsOf,
	START_HANDLE,
	START_ID,
} from '#/pages/survey/lib/survey-graph'
import type { GraphNode } from '#/pages/survey/types'
import { FirstQuestionPlaceholder } from '#/pages/survey/ui/survey-empty-canvas-prompt'

const START_HUE = 250

function regionLabel(node: GraphNode, sentence: string): string {
	if (node.type === 'QUESTION') {
		const label =
			node.label.trim() === '' ? 'Question sans libellé' : node.label
		return `Question ${nodeAppearance(node).typeLabel} : ${label}`
	}
	if (node.type === 'CONDITION') return `Condition : ${sentence}`
	const message = (node.message ?? '').trim()
	return `Fin : ${message || 'message par défaut'}`
}

function subtitleOf(node: GraphNode, sentence: string): string {
	if (node.type === 'QUESTION') {
		const kind = nodeAppearance(node).typeLabel
		const flags = [
			node.required ? 'obligatoire' : null,
			node.identifier ? 'identifiant' : null,
		].filter(Boolean)
		return flags.length > 0 ? `${kind} · ${flags.join(' · ')}` : kind
	}
	if (node.type === 'CONDITION') return sentence
	const message = (node.message ?? '').trim()
	return message || 'Message de fin par défaut'
}

export function SurveyFlowNodeView({
	id,
	data,
	selected,
}: NodeProps<SurveyFlowNode>) {
	const actions = useSurveyCanvasActions()
	const node = data.node
	const editable = !data.readOnly
	const open = new Set<ExitHandle>(editable ? data.openExits : [])

	if (!node) {
		return (
			<section aria-label="Départ du questionnaire">
				<AutomationNodeCard
					label="Départ"
					subtitle="Début du questionnaire"
					icon={Play}
					hue={START_HUE}
					trigger
					status={null}
					needsSetup={false}
					disabled={false}
					bindingIssues={[]}
					selected={false}
					hasInput={false}
					outputs={[{ id: START_HANDLE, open: open.has(START_HANDLE) }]}
					onOpen={() => undefined}
					onAddAfter={() => actions.addAfter(START_ID, START_HANDLE)}
				/>
			</section>
		)
	}

	const appearance = nodeAppearance(node)
	return (
		<section
			aria-label={regionLabel(node, data.sentence)}
			data-highlighted={data.highlighted ? 'true' : undefined}
			className={
				data.highlighted
					? 'rounded-lg shadow-[0_0_0_6px] shadow-destructive/25'
					: undefined
			}
		>
			<AutomationNodeCard
				label={nodeTitle(node)}
				subtitle={subtitleOf(node, data.sentence)}
				icon={appearance.icon}
				hue={appearance.hue}
				trigger={false}
				status={null}
				needsSetup={false}
				disabled={false}
				bindingIssues={data.issues}
				selected={selected}
				hasInput
				outputs={exitsOf(node).map((branch) => ({
					id: branch,
					label: node.type === 'CONDITION' ? BRANCH_LABELS[branch] : undefined,
					open: open.has(branch),
				}))}
				onOpen={() => actions.openNode(id)}
				onAddAfter={(outputId) => {
					const branch = exitsOf(node).find((exit) => exit === outputId)
					if (branch) actions.addAfter(id, branch)
				}}
			/>
		</section>
	)
}

export function SurveyPlaceholderNodeView() {
	const actions = useSurveyCanvasActions()
	return (
		<FirstQuestionPlaceholder
			onAdd={() => actions.addAfter(START_ID, START_HANDLE)}
		/>
	)
}

export const SURVEY_NODE_TYPES = {
	placeholder: SurveyPlaceholderNodeView,
	start: SurveyFlowNodeView,
	question: SurveyFlowNodeView,
	condition: SurveyFlowNodeView,
	end: SurveyFlowNodeView,
}

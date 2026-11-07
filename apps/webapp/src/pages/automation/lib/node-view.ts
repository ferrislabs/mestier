import type { Schemas } from '#/api/api.client'
import { branchLabel } from '#/pages/automation/lib/branch-labels'
import {
	describeConnector,
	triggerCopy,
} from '#/pages/automation/lib/connector-copy'
import {
	outputItems,
	parseStepOutput,
	triggerItem,
} from '#/pages/automation/lib/items'
import { triggerKindOf } from '#/pages/automation/lib/trigger-events'

export interface OutputBranch {
	id: string
	label: string
	items: readonly unknown[]
}

export type NodeOutputView =
	| { state: 'not-executed' }
	| { state: 'error'; message: string; detail?: string }
	| { state: 'ready'; branches: OutputBranch[] }

export type NodeInputView =
	| { state: 'trigger' }
	| { state: 'unconnected' }
	| { state: 'not-executed'; sourceLabels: string[] }
	| { state: 'ready'; sourceLabels: string[]; items: unknown[] }

export function outputBranches(output: unknown): OutputBranch[] {
	const parsed = parseStepOutput(output)
	return parsed.kind === 'main'
		? [{ id: 'main', label: 'Sortie', items: parsed.items }]
		: parsed.lists.map((list) => ({
				id: list.branch,
				label: branchLabel(list.branch),
				items: list.items,
			}))
}

function readyView(output: unknown): NodeOutputView {
	return { state: 'ready', branches: outputBranches(output) }
}

export function outputViewFromStep(
	step: Schemas.RunStepResponse | null | undefined,
): NodeOutputView {
	if (!step) return { state: 'not-executed' }
	if (step.error) return { state: 'error', message: step.error }
	if (step.output === undefined) return { state: 'not-executed' }
	return readyView(step.output)
}

export type DryRunStepView = Pick<
	Schemas.DryRunStepResponse,
	'status' | 'input' | 'output' | 'error'
>

export type ExecutionInputView =
	| { state: 'absent' }
	| { state: 'not-reached' }
	| { state: 'ready'; items: unknown[] }

export function outputViewFromDryStep(
	step: DryRunStepView | undefined,
): NodeOutputView {
	if (!step || step.status === 'skipped') return { state: 'not-executed' }
	if (step.status === 'failed') {
		return {
			state: 'error',
			message: step.error || 'Ce nœud a échoué lors du test.',
		}
	}
	return readyView(step.output)
}

export function inputViewFromDryStep(
	step: DryRunStepView | undefined,
	passesThrough: boolean,
): ExecutionInputView {
	if (!step) return { state: 'absent' }
	if (step.status === 'skipped') return { state: 'not-reached' }
	const input = step.input ?? (passesThrough ? step.output : undefined)
	return { state: 'ready', items: outputItems(input, []) }
}

export function outputViewFromPayload(payload: unknown): NodeOutputView {
	if (payload === undefined || payload === null) {
		return { state: 'not-executed' }
	}
	return {
		state: 'ready',
		branches: [{ id: 'main', label: 'Sortie', items: [triggerItem(payload)] }],
	}
}

interface DescribedNode {
	subtitle: string
	typeLabel: string
	family?: string
	connector?: Schemas.PlacedConnectorDto
	trigger?: Schemas.PlacedTriggerDto
}

function typeSubtitle(node: DescribedNode): string {
	if (node.trigger) {
		return triggerCopy(triggerKindOf(node.trigger.kind)).subtitle
	}
	return describeConnector({
		kind: node.connector?.kind ?? '',
		family: node.family ?? '',
		label: node.typeLabel,
	}).subtitle
}

export function nodeDescription(node: DescribedNode): string {
	return node.subtitle || `${node.typeLabel} · ${typeSubtitle(node)}`
}

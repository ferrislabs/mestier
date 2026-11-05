import type { Schemas } from '#/api/api.client'
import { toItems } from '#/pages/automation/lib/items'

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

function readyView(output: unknown): NodeOutputView {
	return {
		state: 'ready',
		branches: [{ id: 'main', label: 'Sortie', items: toItems(output) }],
	}
}

export function outputViewFromStep(
	step: Schemas.RunStepResponse | null | undefined,
): NodeOutputView {
	if (!step) return { state: 'not-executed' }
	if (step.error) return { state: 'error', message: step.error }
	if (step.output === undefined) return { state: 'not-executed' }
	return readyView(step.output)
}

export function outputViewFromPayload(payload: unknown): NodeOutputView {
	if (payload === undefined || payload === null) {
		return { state: 'not-executed' }
	}
	return readyView(payload)
}

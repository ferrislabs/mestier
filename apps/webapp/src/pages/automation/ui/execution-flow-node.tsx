import type { Node, NodeProps } from '@xyflow/react'
import type { LucideIcon } from 'lucide-react'
import {
	AutomationNodeCard,
	type NodeCardExecution,
	type NodeCardOutput,
} from '#/pages/automation/ui/automation-node-card'

export interface ExecutionNodeData extends Record<string, unknown> {
	label: string
	subtitle: string
	icon: LucideIcon
	hue: number
	trigger: boolean
	outputs: readonly NodeCardOutput[]
	execution: NodeCardExecution
	onOpen: () => void
}

export type ExecutionFlowNodeType = Node<ExecutionNodeData, 'execution'>

const NO_ISSUES: readonly string[] = []

export function ExecutionFlowNode({ data }: NodeProps<ExecutionFlowNodeType>) {
	return (
		<AutomationNodeCard
			label={data.label}
			subtitle={data.subtitle}
			icon={data.icon}
			hue={data.hue}
			trigger={data.trigger}
			status={null}
			needsSetup={false}
			disabled={false}
			bindingIssues={NO_ISSUES}
			selected={false}
			hasInput={!data.trigger}
			outputs={data.outputs}
			execution={data.execution}
			onOpen={data.onOpen}
			onAddAfter={() => {}}
		/>
	)
}

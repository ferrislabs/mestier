import { type NodeProps, useNodeConnections } from '@xyflow/react'
import { useEditorActions } from '#/pages/automation/feature/editor-actions'
import { useNodeIssues } from '#/pages/automation/feature/node-issues-context'
import {
	type EditorNode,
	outputHandles,
} from '#/pages/automation/lib/editor-nodes'
import { executionFromRunStatus } from '#/pages/automation/lib/execution-status'
import {
	appearanceOf,
	needsEventChoice,
} from '#/pages/automation/lib/node-appearance'
import { AutomationNodeCard } from '#/pages/automation/ui/automation-node-card'

export function AutomationFlowNode({
	id,
	data,
	selected,
}: NodeProps<EditorNode>) {
	const actions = useEditorActions()
	const issues = useNodeIssues(id)
	const outgoing = useNodeConnections({ handleType: 'source' })
	const connected = new Set(
		outgoing.map((connection) => connection.sourceHandle ?? null),
	)
	const appearance = appearanceOf(data)
	const trigger = data.kind === 'trigger'
	const needsSetup = needsEventChoice(data)
	const execution = executionFromRunStatus(data.runStatus)

	return (
		<AutomationNodeCard
			label={data.label}
			subtitle={data.subtitle}
			icon={appearance.icon}
			hue={appearance.hue}
			trigger={trigger}
			status={null}
			needsSetup={needsSetup}
			disabled={data.disabled}
			bindingIssues={[
				...new Set([
					...issues.map((issue) => issue.message),
					...data.errors.map((error) => error.message),
				]),
			]}
			selected={selected}
			hasInput={!trigger}
			outputs={outputHandles(data).map((handle) => ({
				id: handle,
				label: handle ?? undefined,
				open: !connected.has(handle),
			}))}
			execution={execution ? { status: execution, note: '' } : null}
			onOpen={() =>
				needsSetup ? actions.chooseEvent(id) : actions.openNode(id)
			}
			onAddAfter={(outputId) => actions.addAfter(id, outputId)}
		/>
	)
}

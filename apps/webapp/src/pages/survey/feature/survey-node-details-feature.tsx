import { useState } from 'react'
import type { NdvNavItem } from '#/pages/automation/ui/ndv-node-nav'
import { NodeDetailsDialog } from '#/pages/automation/ui/node-details-dialog'
import { nodeAppearance, nodeTitle } from '#/pages/survey/lib/node-catalogue'
import {
	BRANCH_LABELS,
	conditionSentence,
} from '#/pages/survey/lib/survey-flow'
import {
	connect,
	precedingChoiceQuestions,
	replaceNode,
	START_HANDLE,
	START_ID,
} from '#/pages/survey/lib/survey-graph'
import {
	type SurveyNeighbour,
	surveyNeighboursOf,
} from '#/pages/survey/lib/survey-neighbours'
import type { EditorGraph, GraphNode } from '#/pages/survey/types'
import {
	NodeDetailsPanes,
	NodeSettings,
} from '#/pages/survey/ui/node-details-panel'
import { RespondentPreview } from '#/pages/survey/ui/respondent-preview'
import { DEFAULT_END_MESSAGE } from '#/pages/survey-respond/feature/use-survey-respondent'

export function SurveyNodeDetailsFeature({
	graph,
	node,
	surveyTitle,
	issues,
	readOnly,
	onGraphChange,
	onNavigate,
	onDelete,
	onClose,
}: {
	graph: EditorGraph
	node: GraphNode
	surveyTitle: string
	issues: readonly string[]
	readOnly: boolean
	onGraphChange: (update: (graph: EditorGraph) => EditorGraph) => void
	onNavigate: (nodeId: string) => void
	onDelete: () => void
	onClose: () => void
}) {
	const [optionKeys, setOptionKeys] = useState<{
		nodeId: string | null
		keys: string[]
	}>({ nodeId: null, keys: [] })

	const byId = new Map(
		graph.nodes.map((candidate) => [candidate.id, candidate]),
	)
	const appearance = nodeAppearance(node)
	const neighbours = surveyNeighboursOf(graph, node.id)
	const sentence =
		node.type === 'CONDITION' ? conditionSentence(graph, node) : ''

	const keysFor = (current: GraphNode): string[] => {
		if (current.type !== 'QUESTION') return []
		const count = current.options?.length ?? 0
		const known = optionKeys.nodeId === current.id ? optionKeys.keys : []
		if (known.length === count) return known
		return Array.from(
			{ length: count },
			(_, position) => known[position] ?? `${current.id}:${position}`,
		)
	}

	const toNavItem =
		(withBranch: boolean) =>
		(neighbour: SurveyNeighbour): NdvNavItem[] => {
			const target = byId.get(neighbour.id)
			if (!target) return []
			const look = nodeAppearance(target)
			return [
				{
					id: target.id,
					label: nodeTitle(target),
					icon: look.icon,
					hue: look.hue,
					branch:
						withBranch && neighbour.branch !== 'NEXT'
							? BRANCH_LABELS[neighbour.branch]
							: undefined,
				},
			]
		}

	const change = (next: GraphNode) => {
		if (next.type === 'QUESTION' && next.options) {
			setOptionKeys({ nodeId: next.id, keys: keysFor(next) })
		}
		onGraphChange((current) => replaceNode(current, next))
	}

	const branches =
		node.type === 'CONDITION'
			? (['THEN', 'ELSE'] as const).map((branch) => {
					const edge = graph.edges.find(
						(candidate) =>
							candidate.from === node.id && candidate.branch === branch,
					)
					const target = edge ? byId.get(edge.to) : undefined
					return {
						branch: BRANCH_LABELS[branch],
						target: target ? nodeTitle(target) : null,
					}
				})
			: []

	return (
		<NodeDetailsDialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
			title={nodeTitle(node)}
			description={appearance.description}
			badge={
				<span className="hidden shrink-0 items-center gap-1.5 sm:inline-flex">
					{graph.start === node.id ? (
						<span className="rounded-sm bg-brand-soft px-1.5 py-0.5 text-primary text-xs">
							Départ
						</span>
					) : null}
					{readOnly ? (
						<span className="rounded-sm border border-border px-1.5 py-0.5 text-muted-foreground text-xs">
							Lecture seule
						</span>
					) : null}
					<span className="rounded-sm border border-border px-1.5 py-0.5 text-muted-foreground text-xs">
						{appearance.typeLabel}
					</span>
				</span>
			}
			icon={appearance.icon}
			hue={appearance.hue}
			status={null}
			input={null}
			parameters={
				<NodeDetailsPanes
					settings={
						<NodeSettings
							key={node.id}
							node={node}
							issues={issues}
							choiceQuestions={precedingChoiceQuestions(graph, node.id)}
							conditionQuestion={
								node.type === 'CONDITION'
									? (byId.get(node.question) ?? null)
									: null
							}
							sentence={sentence}
							optionKeys={keysFor(node)}
							otherIdentifier={graph.nodes.some(
								(candidate) =>
									candidate.type === 'QUESTION' &&
									candidate.id !== node.id &&
									Boolean(candidate.identifier),
							)}
							isStart={graph.start === node.id}
							readOnly={readOnly}
							onChange={change}
							onAddOption={() => {
								if (node.type !== 'QUESTION') return
								setOptionKeys({
									nodeId: node.id,
									keys: [...keysFor(node), crypto.randomUUID()],
								})
								onGraphChange((current) =>
									replaceNode(current, {
										...node,
										options: [...(node.options ?? []), ''],
									}),
								)
							}}
							onRemoveOption={(position) => {
								if (node.type !== 'QUESTION') return
								setOptionKeys({
									nodeId: node.id,
									keys: keysFor(node).filter((_, index) => index !== position),
								})
								onGraphChange((current) =>
									replaceNode(current, {
										...node,
										options: (node.options ?? []).filter(
											(_, index) => index !== position,
										),
									}),
								)
							}}
							onMakeStart={() =>
								onGraphChange((current) =>
									connect(current, START_ID, START_HANDLE, node.id),
								)
							}
							onDelete={onDelete}
						/>
					}
					preview={
						<RespondentPreview
							node={node}
							surveyTitle={surveyTitle}
							endMessage={
								node.type === 'END'
									? node.message?.trim() || DEFAULT_END_MESSAGE
									: ''
							}
							sentence={sentence}
							branches={branches}
						/>
					}
				/>
			}
			output={null}
			previous={neighbours.previous.flatMap(toNavItem(false))}
			next={neighbours.next.flatMap(toNavItem(true))}
			onNavigate={onNavigate}
		/>
	)
}

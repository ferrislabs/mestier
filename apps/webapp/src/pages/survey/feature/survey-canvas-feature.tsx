import '@xyflow/react/dist/style.css'
import '#/pages/automation/ui/automation.css'
import {
	Background,
	BackgroundVariant,
	type Connection,
	ControlButton,
	Controls,
	type EdgeChange,
	type NodeChange,
	ReactFlow,
	ReactFlowProvider,
	useReactFlow,
} from '@xyflow/react'
import { BrushCleaning, Plus } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Button } from '#/components/ui/button'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { GRID_SIZE } from '#/pages/automation/lib/graph'
import {
	buildFlowEdges,
	buildFlowNodes,
	layoutPositions,
	parseEdgeId,
	type SurveyFlowNode,
} from '#/pages/survey/lib/survey-flow'
import {
	addNode,
	connect,
	createNode,
	disconnect,
	type ExitHandle,
	openExit,
	removeNodes,
	START_HANDLE,
	START_ID,
} from '#/pages/survey/lib/survey-graph'
import type { EditorGraph, NodeChoice } from '#/pages/survey/types'
import { NodePickerPanel } from '#/pages/survey/ui/node-picker-panel'
import { SURVEY_NODE_TYPES } from '#/pages/survey/ui/survey-flow-nodes'

const DELETE_KEYS = ['Backspace', 'Delete']
const FIT_VIEW_OPTIONS = { padding: 0.25, maxZoom: 1 }
const ANIMATED_FIT_VIEW_OPTIONS = { ...FIT_VIEW_OPTIONS, duration: 300 }
const NODE_GAP_X = 300
const ARIA_LABELS = {
	'controls.ariaLabel': 'Contrôles du canevas',
	'controls.zoomIn.ariaLabel': 'Zoomer',
	'controls.zoomOut.ariaLabel': 'Dézoomer',
	'controls.fitView.ariaLabel': 'Ajuster la vue',
	'handle.ariaLabel': 'Point de connexion',
	'node.a11yDescription.default':
		'Appuyez sur Entrée ou Espace pour sélectionner un nœud. Supprimez-le avec Suppr.',
	'edge.a11yDescription.default':
		'Appuyez sur Entrée ou Espace pour sélectionner une liaison. Supprimez-la avec Suppr.',
}

export interface SurveyCanvasProps {
	graph: EditorGraph
	readOnly: boolean
	issues: ReadonlyMap<string, string[]>
	highlightedId: string | null
	selectedId: string | null
	nodeLabel: (nodeId: string) => string
	onSelect: (nodeId: string | null) => void
	onGraphChange: (update: (graph: EditorGraph) => EditorGraph) => void
}

export function SurveyCanvasFeature(props: SurveyCanvasProps) {
	return (
		<ReactFlowProvider>
			<SurveyCanvas {...props} />
		</ReactFlowProvider>
	)
}

function SurveyCanvas({
	graph,
	readOnly,
	issues,
	highlightedId,
	selectedId,
	nodeLabel,
	onSelect,
	onGraphChange,
}: SurveyCanvasProps) {
	const { fitView, screenToFlowPosition } = useReactFlow()
	const [positions, setPositions] = useState<Map<string, NodePosition>>(() =>
		layoutPositions(graph, new Map()),
	)
	const [measured, setMeasured] = useState<
		Map<string, { width: number; height: number }>
	>(() => new Map())
	const [selectedEdges, setSelectedEdges] = useState<Set<string>>(
		() => new Set(),
	)
	const [pickerOpen, setPickerOpen] = useState(false)

	const nodes = useMemo(
		() =>
			buildFlowNodes({
				graph,
				positions,
				selectedId,
				issues,
				highlightedId,
				readOnly,
				measured,
			}),
		[graph, positions, selectedId, issues, highlightedId, readOnly, measured],
	)
	const edges = useMemo(
		() =>
			buildFlowEdges(graph, readOnly).map((edge) => ({
				...edge,
				selected: selectedEdges.has(edge.id),
			})),
		[graph, readOnly, selectedEdges],
	)

	const attachment = useMemo(() => {
		const from = selectedId ?? (graph.start ? null : START_ID)
		if (!from) return null
		const handle = openExit(graph, from)
		return handle ? { from, handle } : null
	}, [graph, selectedId])

	const onNodesChange = (changes: NodeChange<SurveyFlowNode>[]) => {
		const moved = new Map(positions)
		const sized = new Map(measured)
		const removed = new Set<string>()
		let positionsChanged = false
		let sizesChanged = false
		for (const change of changes) {
			if (change.type === 'position' && change.position) {
				moved.set(change.id, change.position)
				positionsChanged = true
			} else if (change.type === 'dimensions' && change.dimensions) {
				sized.set(change.id, change.dimensions)
				sizesChanged = true
			} else if (
				change.type === 'remove' &&
				change.id !== START_ID &&
				!readOnly
			) {
				removed.add(change.id)
			}
		}
		if (positionsChanged) setPositions(moved)
		if (sizesChanged) setMeasured(sized)
		if (removed.size > 0) {
			if (selectedId && removed.has(selectedId)) onSelect(null)
			onGraphChange((current) => removeNodes(current, removed))
		}
	}

	const onEdgesChange = (changes: EdgeChange[]) => {
		const selection = new Set(selectedEdges)
		const removed: { from: string; handle: ExitHandle }[] = []
		for (const change of changes) {
			if (change.type === 'select') {
				if (change.selected) selection.add(change.id)
				else selection.delete(change.id)
			} else if (change.type === 'remove' && !readOnly) {
				const parsed = parseEdgeId(change.id)
				if (parsed) removed.push(parsed)
				selection.delete(change.id)
			}
		}
		setSelectedEdges(selection)
		if (removed.length > 0) {
			onGraphChange((current) =>
				removed.reduce(
					(next, edge) => disconnect(next, edge.from, edge.handle),
					current,
				),
			)
		}
	}

	const onConnect = (connection: Connection) => {
		if (readOnly || !connection.sourceHandle) return
		const handle = connection.sourceHandle as ExitHandle
		onGraphChange((current) =>
			connect(current, connection.source, handle, connection.target),
		)
	}

	const pick = (choice: NodeChoice) => {
		const id = crypto.randomUUID()
		const node = createNode(choice, id)
		const anchor = attachment ? positions.get(attachment.from) : undefined
		const position = anchor
			? { x: anchor.x + NODE_GAP_X, y: anchor.y }
			: screenToFlowPosition({
					x: window.innerWidth / 2,
					y: window.innerHeight / 2,
				})
		setPositions(new Map(positions).set(id, position))
		const after =
			attachment &&
			!(attachment.handle === START_HANDLE && node.type !== 'QUESTION')
				? attachment
				: null
		onGraphChange((current) => addNode(current, node, after))
		onSelect(id)
		setPickerOpen(false)
	}

	const tidyUp = () => {
		setPositions(layoutPositions(graph, positions))
		requestAnimationFrame(() => {
			void fitView(ANIMATED_FIT_VIEW_OPTIONS)
		})
	}

	return (
		<div className="relative min-h-0 w-full flex-1">
			<ReactFlow
				className="automation-flow"
				nodes={nodes}
				edges={edges}
				nodeTypes={SURVEY_NODE_TYPES}
				onNodesChange={onNodesChange}
				onEdgesChange={onEdgesChange}
				onConnect={onConnect}
				isValidConnection={(connection) =>
					connection.source !== connection.target &&
					connection.target !== START_ID
				}
				onNodeClick={(_, node) =>
					onSelect(node.id === START_ID ? null : node.id)
				}
				onPaneClick={() => onSelect(null)}
				nodesConnectable={!readOnly}
				deleteKeyCode={readOnly ? null : DELETE_KEYS}
				fitView
				fitViewOptions={FIT_VIEW_OPTIONS}
				minZoom={0.25}
				maxZoom={1.5}
				zoomOnDoubleClick={false}
				snapToGrid
				snapGrid={[GRID_SIZE, GRID_SIZE]}
				ariaLabelConfig={ARIA_LABELS}
			>
				<Background
					variant={BackgroundVariant.Dots}
					gap={GRID_SIZE}
					size={1.6}
				/>
				<Controls
					position="bottom-left"
					showInteractive={false}
					fitViewOptions={ANIMATED_FIT_VIEW_OPTIONS}
				>
					<ControlButton
						onClick={tidyUp}
						title="Réorganiser le canevas"
						aria-label="Réorganiser le canevas"
					>
						<BrushCleaning />
					</ControlButton>
				</Controls>
			</ReactFlow>
			{graph.nodes.length === 0 && !readOnly && !pickerOpen ? (
				<div className="pointer-events-none absolute inset-0 flex items-center justify-center">
					<div className="pointer-events-auto rounded-xl border bg-card p-6 text-center shadow-sm">
						<p className="font-medium">Le questionnaire est vide</p>
						<p className="mt-1 text-muted-foreground text-sm">
							Ajoutez la première question : elle sera reliée au départ.
						</p>
						<Button className="mt-4" onClick={() => setPickerOpen(true)}>
							<Plus aria-hidden />
							Ajouter une question
						</Button>
					</div>
				</div>
			) : null}
			{!readOnly && !pickerOpen ? (
				<Button
					variant="outline"
					size="icon"
					className="absolute top-3 right-3 z-10 bg-card"
					aria-label="Ajouter un nœud"
					onClick={() => setPickerOpen(true)}
				>
					<Plus aria-hidden />
				</Button>
			) : null}
			{pickerOpen ? (
				<NodePickerPanel
					attachedTo={
						attachment
							? attachment.from === START_ID
								? 'Départ'
								: nodeLabel(attachment.from)
							: null
					}
					onPick={pick}
					onClose={() => setPickerOpen(false)}
				/>
			) : null}
		</div>
	)
}

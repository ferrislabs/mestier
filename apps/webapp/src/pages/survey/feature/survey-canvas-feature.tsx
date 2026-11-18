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
import {
	type MouseEvent as ReactMouseEvent,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react'
import { Button } from '#/components/ui/button'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { GRID_SIZE, NODE_SIZE } from '#/pages/automation/lib/graph'
import {
	canvasShortcutFor,
	isMacPlatform,
	shortcutLabels,
} from '#/pages/automation/lib/shortcuts'
import { PaneMenuItems } from '#/pages/automation/ui/canvas-menu-items'
import {
	ContextMenuAt,
	type MenuAnchor,
} from '#/pages/automation/ui/context-menu-at'
import {
	type SurveyCanvasActions,
	SurveyCanvasActionsContext,
} from '#/pages/survey/feature/survey-canvas-actions'
import { SURVEY_NODE_TYPES } from '#/pages/survey/feature/survey-flow-node'
import { SurveyNodeCreatorFeature } from '#/pages/survey/feature/survey-node-creator-feature'
import {
	BRANCH_LABELS,
	buildFlowEdges,
	buildFlowNodes,
	layoutPositions,
	PLACEHOLDER_ID,
	parseEdgeId,
	type SurveyFlowNode,
	withFirstQuestionPlaceholder,
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
import { SurveyNodeMenuItems } from '#/pages/survey/ui/survey-canvas-menu-items'

const DELETE_KEYS = ['Backspace', 'Delete']
const FIT_VIEW_OPTIONS = { padding: 0.3, maxZoom: 1 }
const ANIMATED_FIT_VIEW_OPTIONS = { ...FIT_VIEW_OPTIONS, duration: 300 }
const NODE_GAP_X = 220
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

type Attachment = { from: string; handle: ExitHandle }

interface CreatorState {
	key: number
	attachment: Attachment | null
	position: NodePosition | null
}

interface CanvasMenu {
	anchor: MenuAnchor
	position: NodePosition
	nodeId: string | null
}

export interface SurveyCanvasProps {
	graph: EditorGraph
	readOnly: boolean
	issues: ReadonlyMap<string, string[]>
	highlightedId: string | null
	selectedId: string | null
	focusRequest: { nodeId: string; key: number } | null
	nodeLabel: (nodeId: string) => string
	onSelect: (nodeId: string | null) => void
	onOpenNode: (nodeId: string) => void
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
	focusRequest,
	nodeLabel,
	onSelect,
	onOpenNode,
	onGraphChange,
}: SurveyCanvasProps) {
	const { fitView, screenToFlowPosition } = useReactFlow()
	const containerRef = useRef<HTMLDivElement>(null)
	const [positions, setPositions] = useState<Map<string, NodePosition>>(() =>
		layoutPositions(graph, new Map()),
	)
	const [measured, setMeasured] = useState<
		Map<string, { width: number; height: number }>
	>(() => new Map())
	const [selectedEdges, setSelectedEdges] = useState<Set<string>>(
		() => new Set(),
	)
	const [creator, setCreator] = useState<CreatorState | null>(null)
	const [menu, setMenu] = useState<CanvasMenu | null>(null)
	const shortcuts = useMemo(
		() => shortcutLabels(isMacPlatform(navigator.userAgent)),
		[],
	)

	const showPlaceholder = graph.nodes.length === 0 && !readOnly
	const { nodes, edges } = useMemo(() => {
		const built = buildFlowNodes({
			graph,
			positions,
			selectedId,
			issues,
			highlightedId,
			readOnly,
			measured,
		})
		const linked = buildFlowEdges(graph, readOnly).map((edge) => ({
			...edge,
			selected: selectedEdges.has(edge.id),
		}))
		return showPlaceholder
			? withFirstQuestionPlaceholder(
					built,
					linked,
					positions.get(START_ID) ?? { x: 0, y: 0 },
					measured.get(PLACEHOLDER_ID),
				)
			: { nodes: built, edges: linked }
	}, [
		graph,
		positions,
		selectedId,
		issues,
		highlightedId,
		readOnly,
		measured,
		selectedEdges,
		showPlaceholder,
	])

	const openCreator = (
		attachment: Attachment | null,
		position: NodePosition | null = null,
	) => {
		if (readOnly) return
		setCreator((current) => ({
			key: (current?.key ?? 0) + 1,
			attachment,
			position,
		}))
	}

	const toolbarAttachment = (): Attachment | null => {
		const from = selectedId ?? (graph.start ? null : START_ID)
		if (!from) return null
		const handle = openExit(graph, from)
		return handle ? { from, handle } : null
	}

	const actions = useMemo<SurveyCanvasActions>(
		() => ({
			openNode: onOpenNode,
			addAfter: (from, handle) => {
				if (readOnly) return
				setCreator((current) => ({
					key: (current?.key ?? 0) + 1,
					attachment: { from, handle },
					position: null,
				}))
			},
		}),
		[onOpenNode, readOnly],
	)

	const focusKey = focusRequest?.key
	const focusId = focusRequest?.nodeId
	useEffect(() => {
		if (focusKey === undefined || !focusId) return
		requestAnimationFrame(() => {
			void fitView({ ...ANIMATED_FIT_VIEW_OPTIONS, nodes: [{ id: focusId }] })
		})
	}, [focusKey, focusId, fitView])

	const shortcutRef = useRef<(event: KeyboardEvent) => void>(() => undefined)
	shortcutRef.current = (event: KeyboardEvent) => {
		if (canvasShortcutFor(event) !== 'open' || !selectedId) return
		const target = event.target instanceof Element ? event.target : null
		if (
			target?.closest('input, textarea, select, [contenteditable="true"]') ||
			(target?.closest('button, a') && !target.closest('.react-flow__node')) ||
			document.querySelector('[role="dialog"], [role="menu"]')
		)
			return
		event.preventDefault()
		onOpenNode(selectedId)
	}
	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => shortcutRef.current(event)
		window.addEventListener('keydown', onKeyDown)
		return () => window.removeEventListener('keydown', onKeyDown)
	}, [])

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
				change.id !== PLACEHOLDER_ID &&
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
		const removed: Attachment[] = []
		for (const change of changes) {
			if (change.type === 'select') {
				if (change.selected) selection.add(change.id)
				else selection.delete(change.id)
			} else if (change.type === 'remove' && !readOnly) {
				const parsed = parseEdgeId(change.id)
				if (parsed) removed.push({ from: parsed.from, handle: parsed.handle })
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
		if (!creator) return
		const id = crypto.randomUUID()
		const node = createNode(choice, id)
		const attachment = creator.attachment
		const anchor = attachment ? positions.get(attachment.from) : undefined
		const position = anchor
			? { x: anchor.x + NODE_GAP_X, y: anchor.y }
			: creator.position
				? {
						x: creator.position.x - NODE_SIZE / 2,
						y: creator.position.y - NODE_SIZE / 2,
					}
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
		setCreator(null)
		if (node.type !== 'END') onOpenNode(id)
	}

	const tidyUp = () => {
		setPositions(layoutPositions(graph, positions))
		requestAnimationFrame(() => {
			void fitView(ANIMATED_FIT_VIEW_OPTIONS)
		})
	}

	const openCanvasMenu = (
		event: MouseEvent | ReactMouseEvent,
		nodeId: string | null,
	) => {
		event.preventDefault()
		if (readOnly && !nodeId) return
		const bounds = containerRef.current?.getBoundingClientRect()
		if (!bounds) return
		setMenu({
			anchor: { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
			position: screenToFlowPosition({ x: event.clientX, y: event.clientY }),
			nodeId,
		})
	}

	const menuNode = graph.nodes.find((node) => node.id === menu?.nodeId)
	const attachedTo = creator?.attachment
		? creator.attachment.from === START_ID
			? 'Départ'
			: `${nodeLabel(creator.attachment.from)}${
					creator.attachment.handle === 'THEN' ||
					creator.attachment.handle === 'ELSE'
						? ` (${BRANCH_LABELS[creator.attachment.handle]})`
						: ''
				}`
		: null

	return (
		<SurveyCanvasActionsContext value={actions}>
			<div ref={containerRef} className="relative min-h-0 w-full flex-1">
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
						onSelect(
							node.id === START_ID || node.id === PLACEHOLDER_ID
								? null
								: node.id,
						)
					}
					onNodeDoubleClick={(_, node) => {
						if (node.id !== START_ID && node.id !== PLACEHOLDER_ID)
							onOpenNode(node.id)
					}}
					onPaneClick={() => onSelect(null)}
					onPaneContextMenu={(event) => openCanvasMenu(event, null)}
					onNodeContextMenu={(event, node) => {
						if (node.id === START_ID || node.id === PLACEHOLDER_ID) {
							openCanvasMenu(event, null)
							return
						}
						onSelect(node.id)
						openCanvasMenu(event, node.id)
					}}
					onMoveStart={() => setMenu(null)}
					nodesConnectable={!readOnly}
					nodesFocusable={false}
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
				{!readOnly && !creator ? (
					<Button
						variant="outline"
						size="icon"
						className="absolute top-3 right-3 z-10 bg-card"
						aria-label="Ajouter un nœud"
						title="Ajouter un nœud"
						onClick={() => openCreator(toolbarAttachment())}
					>
						<Plus aria-hidden />
					</Button>
				) : null}
				<ContextMenuAt
					anchor={menu?.anchor ?? null}
					label={
						menuNode
							? `Actions sur « ${nodeLabel(menuNode.id)} »`
							: 'Actions du canevas'
					}
					onClose={() => setMenu(null)}
				>
					{menuNode ? (
						<SurveyNodeMenuItems
							readOnly={readOnly}
							canMakeStart={
								menuNode.type === 'QUESTION' && graph.start !== menuNode.id
							}
							shortcuts={shortcuts}
							onOpen={() => onOpenNode(menuNode.id)}
							onMakeStart={() =>
								onGraphChange((current) =>
									connect(current, START_ID, START_HANDLE, menuNode.id),
								)
							}
							onDelete={() => {
								if (selectedId === menuNode.id) onSelect(null)
								onGraphChange((current) =>
									removeNodes(current, new Set([menuNode.id])),
								)
							}}
						/>
					) : (
						<PaneMenuItems
							pasteLabel={null}
							shortcuts={shortcuts}
							onAddNode={() => {
								if (menu) openCreator(null, menu.position)
							}}
							onPaste={() => undefined}
						/>
					)}
				</ContextMenuAt>
				{creator ? (
					<SurveyNodeCreatorFeature
						key={creator.key}
						attachedTo={attachedTo}
						onPick={pick}
						onClose={() => setCreator(null)}
					/>
				) : null}
			</div>
		</SurveyCanvasActionsContext>
	)
}

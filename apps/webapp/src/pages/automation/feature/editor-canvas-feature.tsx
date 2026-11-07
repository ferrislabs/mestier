import '@xyflow/react/dist/style.css'
import '#/pages/automation/ui/automation.css'
import {
	Background,
	BackgroundVariant,
	ControlButton,
	Controls,
	ReactFlow,
	ReactFlowProvider,
	useReactFlow,
} from '@xyflow/react'
import {
	BrushCleaning,
	FlaskConical,
	LoaderCircle,
	Plus,
	Redo2,
	Undo2,
} from 'lucide-react'
import {
	type MouseEvent as ReactMouseEvent,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type CredentialKind,
	useCollectionDetails,
} from '#/hooks/use-automation'
import { AutomationFlowNode } from '#/pages/automation/feature/automation-flow-node'
import {
	type EditorActions,
	EditorActionsContext,
} from '#/pages/automation/feature/editor-actions'
import { EventPickerFeature } from '#/pages/automation/feature/event-picker-feature'
import {
	type CreatorChoice,
	NodeCreatorFeature,
} from '#/pages/automation/feature/node-creator-feature'
import { NodeDetailsFeature } from '#/pages/automation/feature/node-details-feature'
import { NodeIssuesContext } from '#/pages/automation/feature/node-issues-context'
import {
	type GraphChange,
	useWorkflowGraph,
} from '#/pages/automation/feature/use-workflow-graph'
import { referencedCollectionIds } from '#/pages/automation/lib/collection-node'
import {
	type EditorLookup,
	type EditorNode,
	findOpenOutput,
	outputHandles,
	toGraph,
} from '#/pages/automation/lib/editor-nodes'
import {
	copyLabel,
	GRID_SIZE,
	isBranchDeclared,
	NODE_SIZE,
	type NodePosition,
	reconnectionsAfterDelete,
} from '#/pages/automation/lib/graph'
import { tidyLayout } from '#/pages/automation/lib/layout'
import { needsEventChoice } from '#/pages/automation/lib/node-appearance'
import { nodeIssuesByNode } from '#/pages/automation/lib/node-issues'
import { labelConflict } from '#/pages/automation/lib/references'
import {
	type CanvasShortcut,
	canvasShortcutFor,
	disableToggleFor,
	isMacPlatform,
	shortcutLabels,
} from '#/pages/automation/lib/shortcuts'
import {
	type LastRunData,
	nodeOutput,
	type UpstreamSources,
} from '#/pages/automation/lib/upstream-entries'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import type { DryRunPasses } from '#/pages/automation/lib/workflow-runs'
import {
	NodeMenuItems,
	PaneMenuItems,
} from '#/pages/automation/ui/canvas-menu-items'
import {
	ContextMenuAt,
	type MenuAnchor,
} from '#/pages/automation/ui/context-menu-at'
import { EmptyCanvasPrompt } from '#/pages/automation/ui/empty-canvas-prompt'
import { RenameNodeDialog } from '#/pages/automation/ui/rename-node-dialog'
import type { TestRun } from '#/pages/automation/ui/variables-panel'

const NODE_TYPES = { automation: AutomationFlowNode }

const SNAP_GRID: [number, number] = [GRID_SIZE, GRID_SIZE]

const DELETE_KEYS = ['Backspace', 'Delete']

const FIT_VIEW_OPTIONS = { padding: 0.3, maxZoom: 1 }
const ANIMATED_FIT_VIEW_OPTIONS = { ...FIT_VIEW_OPTIONS, duration: 300 }

const ARIA_LABELS = {
	'controls.ariaLabel': 'Contrôles du canevas',
	'controls.zoomIn.ariaLabel': 'Zoomer',
	'controls.zoomOut.ariaLabel': 'Dézoomer',
	'controls.fitView.ariaLabel': 'Ajuster la vue',
	'handle.ariaLabel': 'Point de connexion',
	'node.a11yDescription.default':
		'Appuyez sur Entrée ou Espace pour sélectionner un nœud. Supprimez-le avec Suppr.',
	'edge.a11yDescription.default':
		'Appuyez sur Entrée ou Espace pour sélectionner une connexion. Supprimez-la avec Suppr.',
}

type CreatedCredential = Schemas.CredentialResponse & { secret: unknown }

export type { LastRunData }

export interface WorkflowCanvasProps {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	organizationSlug: string
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors: Map<string, ConnectorValidationError[]>
	events: Schemas.EventDescriptorResponse[]
	lastRun: LastRunData | null
	runStatuses: Map<string, string>
	runSteps: Map<string, Schemas.RunStepResponse>
	dryRunSteps: DryRunPasses | null
	onEvaluateExpression: (
		template: unknown,
		context: Schemas.EvaluateContextBody,
	) => Promise<unknown>
	credentials?: Schemas.CredentialResponse[]
	credentialKinds?: CredentialKind[]
	onCreateCredential?: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<CreatedCredential>
	testRun: TestRun
	canTest?: boolean
	onChange: (change: GraphChange) => void
}

function rejectCreateCredential(): Promise<CreatedCredential> {
	return Promise.reject(
		new Error('onCreateCredential was not wired for this canvas'),
	)
}

interface CreatorState {
	key: number
	open: boolean
	source: { nodeId: string; outputId: string | null } | null
	position: NodePosition | null
}

interface CanvasMenu {
	anchor: MenuAnchor
	position: NodePosition
	nodeId: string | null
}

export function WorkflowCanvas(props: WorkflowCanvasProps) {
	return (
		<ReactFlowProvider>
			<EditorCanvas {...props} />
		</ReactFlowProvider>
	)
}

function EditorCanvas({
	graph: initialGraph,
	layout,
	organizationSlug,
	descriptors,
	connectorErrors,
	events,
	lastRun,
	runStatuses,
	runSteps,
	dryRunSteps,
	onEvaluateExpression,
	credentials = [],
	credentialKinds = [],
	onCreateCredential = rejectCreateCredential,
	testRun,
	canTest = true,
	onChange,
}: WorkflowCanvasProps) {
	const { activeOrganization } = useActiveOrganization()
	const { screenToFlowPosition, deleteElements, fitView } = useReactFlow()
	const containerRef = useRef<HTMLDivElement>(null)
	const lookup = useMemo<EditorLookup>(
		() => ({ descriptors, events }),
		[descriptors, events],
	)
	const graph = useWorkflowGraph({
		graph: initialGraph,
		layout,
		lookup,
		errors: connectorErrors,
		runStatuses,
		onChange,
	})
	const { syncExternal } = graph

	useEffect(() => {
		syncExternal(connectorErrors, runStatuses)
	}, [connectorErrors, runStatuses, syncExternal])

	const [creator, setCreator] = useState<CreatorState>({
		key: 0,
		open: false,
		source: null,
		position: null,
	})
	const [menu, setMenu] = useState<CanvasMenu | null>(null)
	const [fitOnOpen] = useState(() => graph.nodes.length > 0)
	const [openNodeId, setOpenNodeId] = useState<string | null>(null)
	const [eventNodeId, setEventNodeId] = useState<string | null>(null)
	const [renameNodeId, setRenameNodeId] = useState<string | null>(null)
	const [clipboard, setClipboard] = useState<EditorNode | null>(null)

	const sources = useMemo<UpstreamSources>(
		() => ({ descriptors, events, lastRun, runSteps }),
		[descriptors, events, lastRun, runSteps],
	)

	const openCreator = useCallback(
		(source: CreatorState['source'], position: NodePosition | null = null) => {
			setCreator((current) => ({
				key: current.key + 1,
				open: true,
				source,
				position,
			}))
		},
		[],
	)

	const closeCreator = useCallback(
		() =>
			setCreator((current) => ({
				...current,
				open: false,
				source: null,
				position: null,
			})),
		[],
	)

	const actions = useMemo<EditorActions>(
		() => ({
			openNode: setOpenNodeId,
			chooseEvent: setEventNodeId,
			addAfter: (nodeId, outputId) => openCreator({ nodeId, outputId }),
		}),
		[openCreator],
	)

	const openCreatorFromToolbar = () => {
		const selected = graph.nodes.find((node) => node.selected)
		openCreator(selected ? findOpenOutput(selected, graph.edges) : null)
	}

	const createNode = (choice: CreatorChoice) => {
		const anchor =
			creator.position ??
			screenToFlowPosition({
				x: window.innerWidth / 2,
				y: window.innerHeight / 2,
			})
		const fallback = {
			x: anchor.x - NODE_SIZE / 2,
			y: anchor.y - NODE_SIZE / 2,
		}
		if (choice.type === 'trigger') graph.addTrigger(choice.kind, fallback)
		else
			graph.addConnector(
				choice.descriptor,
				creator.source,
				fallback,
				choice.preset,
			)
		closeCreator()
	}

	const openCanvasMenu = (
		event: MouseEvent | ReactMouseEvent,
		nodeId: string | null = null,
	) => {
		event.preventDefault()
		const bounds = containerRef.current?.getBoundingClientRect()
		if (!bounds) return
		setMenu({
			anchor: { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
			position: screenToFlowPosition({ x: event.clientX, y: event.clientY }),
			nodeId,
		})
	}

	const copyNode = (node: EditorNode) => {
		setClipboard(node)
		const payload = {
			kind: node.data.connector?.kind ?? 'trigger',
			label: node.data.label,
			config: node.data.connector?.config ?? node.data.trigger?.kind,
		}
		navigator.clipboard
			?.writeText(JSON.stringify(payload, null, 2))
			.catch(() => undefined)
	}

	const pasteNode = (position: NodePosition) => {
		if (!clipboard) return
		graph.insertCopy(
			clipboard,
			{ x: position.x - NODE_SIZE / 2, y: position.y - NODE_SIZE / 2 },
			clipboard.data.label,
		)
	}

	const tidyUp = () => {
		graph.applyPositions(
			tidyLayout(
				graph.nodes.map((node) => ({
					id: node.id,
					position: node.position,
					trigger: node.data.kind === 'trigger',
					outputs: outputHandles(node.data).map((handle) => handle ?? ''),
				})),
				graph.edges,
			),
		)
		requestAnimationFrame(() => {
			fitView(ANIMATED_FIT_VIEW_OPTIONS)
		})
	}

	const duplicateNode = (node: EditorNode) => {
		graph.insertCopy(
			node,
			{
				x: node.position.x + GRID_SIZE * 2,
				y: node.position.y + GRID_SIZE * 2,
			},
			copyLabel(node.data.label),
		)
	}

	const shortcuts = useMemo(
		() => shortcutLabels(isMacPlatform(navigator.userAgent)),
		[],
	)
	const pointerRef = useRef<{ x: number; y: number } | null>(null)

	const runShortcut = (shortcut: CanvasShortcut): boolean => {
		const selected = graph.nodes.filter((node) => node.selected)
		const single = selected.length === 1 ? selected[0] : null
		switch (shortcut) {
			case 'undo':
				graph.undo()
				return true
			case 'redo':
				graph.redo()
				return true
			case 'copy':
				if (!single) return false
				copyNode(single)
				return true
			case 'paste': {
				if (!clipboard) return false
				const pointer = pointerRef.current
				pasteNode(
					screenToFlowPosition(
						pointer ?? { x: window.innerWidth / 2, y: window.innerHeight / 2 },
					),
				)
				return true
			}
			case 'duplicate':
				if (!single) return false
				duplicateNode(single)
				return true
			case 'rename':
				if (!single) return false
				setRenameNodeId(single.id)
				return true
			case 'open':
				if (!single) return false
				if (needsEventChoice(single.data)) setEventNodeId(single.id)
				else setOpenNodeId(single.id)
				return true
			case 'toggle-disabled': {
				const toggle = disableToggleFor(
					selected.map((node) => ({
						id: node.id,
						kind: node.data.kind,
						disabled: node.data.disabled,
					})),
				)
				if (!toggle) return false
				graph.setDisabled(toggle.ids, toggle.disabled)
				return true
			}
			case 'delete':
				return false
		}
	}
	useEffect(() => {
		const container = containerRef.current
		if (!container) return
		const track = (event: PointerEvent) => {
			pointerRef.current = { x: event.clientX, y: event.clientY }
		}
		container.addEventListener('pointermove', track)
		return () => container.removeEventListener('pointermove', track)
	}, [])

	const runShortcutRef = useRef(runShortcut)
	runShortcutRef.current = runShortcut

	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			const shortcut = canvasShortcutFor(event)
			if (!shortcut) return
			const target = event.target instanceof Element ? event.target : null
			const modified = event.metaKey || event.ctrlKey
			if (
				target?.closest('input, textarea, select, [contenteditable="true"]') ||
				(!modified &&
					target?.closest('button, a') &&
					!target.closest('.react-flow__node')) ||
				document.querySelector('[role="dialog"], [role="menu"]') ||
				(shortcut === 'copy' &&
					(window.getSelection()?.toString() ?? '') !== '')
			)
				return
			if (runShortcutRef.current(shortcut)) event.preventDefault()
		}
		window.addEventListener('keydown', onKeyDown)
		return () => window.removeEventListener('keydown', onKeyDown)
	}, [])

	const referencedIds = useMemo(
		() =>
			referencedCollectionIds(
				graph.nodes.flatMap((node) =>
					node.data.connector ? [node.data.connector] : [],
				),
			),
		[graph.nodes],
	)
	const { reads: collectionReads } = useCollectionDetails(
		activeOrganization.id,
		referencedIds,
	)

	const issuesByNode = useMemo(
		() =>
			nodeIssuesByNode(
				graph.nodes,
				graph.edges,
				(nodeId) => {
					const node = graph.nodes.find((candidate) => candidate.id === nodeId)
					return node ? nodeOutput(node, sources) : undefined
				},
				collectionReads,
			),
		[graph.nodes, graph.edges, sources, collectionReads],
	)

	const hasTrigger = graph.nodes.some((node) => node.data.kind === 'trigger')

	const detailsNode = graph.nodes.find((node) => node.id === openNodeId)
	const eventNode = graph.nodes.find((node) => node.id === eventNodeId)
	const renameNode = graph.nodes.find((node) => node.id === renameNodeId)
	const menuNode = graph.nodes.find((node) => node.id === menu?.nodeId)

	return (
		<EditorActionsContext value={actions}>
			<NodeIssuesContext value={issuesByNode}>
				<div ref={containerRef} className="relative min-h-0 w-full flex-1">
					<ReactFlow
						className="automation-flow"
						nodes={graph.nodes}
						edges={graph.edges}
						nodeTypes={NODE_TYPES}
						onNodesChange={graph.onNodesChange}
						onEdgesChange={graph.onEdgesChange}
						onConnect={graph.onConnect}
						isValidConnection={(connection) =>
							isBranchDeclared(
								connection,
								toGraph(graph.nodes, graph.edges),
								descriptors,
							)
						}
						onNodeClick={(_, node) => {
							if (needsEventChoice(node.data)) setEventNodeId(node.id)
						}}
						onNodeDoubleClick={(_, node) => {
							if (!needsEventChoice(node.data)) setOpenNodeId(node.id)
						}}
						deleteKeyCode={DELETE_KEYS}
						onBeforeDelete={async () => {
							graph.record()
							return true
						}}
						onNodeDragStart={() => graph.record()}
						onDelete={({ nodes: deletedNodes, edges: deletedEdges }) => {
							const deletedEdgeIds = new Set(
								deletedEdges.map((edge) => edge.id),
							)
							graph.reconnect(
								reconnectionsAfterDelete(
									deletedNodes.map((node) => node.id),
									deletedEdges,
									graph.edges.filter((edge) => !deletedEdgeIds.has(edge.id)),
								),
							)
						}}
						onPaneContextMenu={(event) => openCanvasMenu(event)}
						onNodeContextMenu={(event, node) => {
							graph.selectNode(node.id)
							openCanvasMenu(event, node.id)
						}}
						onMoveStart={() => setMenu(null)}
						fitView={fitOnOpen}
						fitViewOptions={FIT_VIEW_OPTIONS}
						minZoom={0.25}
						maxZoom={1.5}
						nodesFocusable={false}
						zoomOnDoubleClick={false}
						snapToGrid
						snapGrid={SNAP_GRID}
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
								onClick={graph.undo}
								disabled={!graph.canUndo}
								title={`Annuler (${shortcuts.undo})`}
								aria-label="Annuler la dernière action"
							>
								<Undo2 />
							</ControlButton>
							<ControlButton
								onClick={graph.redo}
								disabled={!graph.canRedo}
								title={`Rétablir (${shortcuts.redo})`}
								aria-label="Rétablir l’action annulée"
							>
								<Redo2 />
							</ControlButton>
							<ControlButton
								onClick={tidyUp}
								disabled={graph.nodes.length === 0}
								title="Ranger le workflow"
								aria-label="Ranger le workflow"
							>
								<BrushCleaning />
							</ControlButton>
						</Controls>
					</ReactFlow>
					{graph.nodes.length === 0 && (
						<EmptyCanvasPrompt onAdd={() => openCreator(null)} />
					)}
					{!creator.open && (
						<Button
							variant="outline"
							size="icon"
							className="absolute top-3 right-3 z-10 bg-card"
							aria-label="Ajouter un nœud"
							onClick={openCreatorFromToolbar}
						>
							<Plus />
						</Button>
					)}
					{canTest && (
						<div className="absolute bottom-6 left-1/2 z-10 -translate-x-1/2">
							<Button
								size="lg"
								onClick={testRun.start}
								disabled={testRun.running || graph.nodes.length === 0}
							>
								{testRun.running ? (
									<LoaderCircle className="animate-spin" />
								) : (
									<FlaskConical />
								)}
								{testRun.running ? 'Exécution en cours…' : 'Tester le workflow'}
							</Button>
						</div>
					)}
					<ContextMenuAt
						anchor={menu?.anchor ?? null}
						label={
							menuNode
								? `Actions sur « ${menuNode.data.label} »`
								: 'Actions du canevas'
						}
						onClose={() => setMenu(null)}
					>
						{menuNode ? (
							<NodeMenuItems
								canToggleDisabled={menuNode.data.kind === 'connector'}
								disabled={menuNode.data.disabled}
								shortcuts={shortcuts}
								onOpen={() => setOpenNodeId(menuNode.id)}
								onRename={() => setRenameNodeId(menuNode.id)}
								onToggleDisabled={() =>
									graph.setDisabled([menuNode.id], !menuNode.data.disabled)
								}
								onCopy={() => copyNode(menuNode)}
								onDuplicate={() => duplicateNode(menuNode)}
								onDelete={() =>
									deleteElements({ nodes: [{ id: menuNode.id }] })
								}
							/>
						) : (
							<PaneMenuItems
								pasteLabel={clipboard?.data.label ?? null}
								shortcuts={shortcuts}
								onAddNode={() => {
									if (menu) openCreator(null, menu.position)
								}}
								onPaste={() => {
									if (menu) pasteNode(menu.position)
								}}
							/>
						)}
					</ContextMenuAt>
					{creator.open && (
						<NodeCreatorFeature
							key={creator.key}
							organizationSlug={organizationSlug}
							descriptors={[...descriptors.values()]}
							hasTrigger={hasTrigger}
							onCreate={createNode}
							onClose={closeCreator}
						/>
					)}
					{detailsNode && (
						<NodeDetailsFeature
							node={detailsNode}
							nodes={graph.nodes}
							edges={graph.edges}
							organizationSlug={organizationSlug}
							sources={sources}
							collectionReads={collectionReads}
							credentialKinds={credentialKinds}
							credentials={credentials}
							issues={issuesByNode.get(detailsNode.id) ?? []}
							dryRunSteps={dryRunSteps}
							testRun={canTest ? testRun : null}
							onUpdateConnector={(patch) =>
								graph.updateConnector(detailsNode.id, patch)
							}
							onUpdateTriggerKind={(kind) =>
								graph.updateTriggerKind(detailsNode.id, kind)
							}
							onUpdateDetails={(patch) =>
								graph.updateDetails(detailsNode.id, patch)
							}
							onCreateCredential={onCreateCredential}
							onEvaluateExpression={onEvaluateExpression}
							onClose={() => setOpenNodeId(null)}
							onNavigate={(nodeId) => {
								graph.selectNode(nodeId)
								setOpenNodeId(nodeId)
							}}
						/>
					)}
					{renameNode && (
						<RenameNodeDialog
							initialName={renameNode.data.label}
							validate={(name) =>
								labelConflict(name, renameNode.id, graph.nodes)
							}
							onCancel={() => setRenameNodeId(null)}
							onConfirm={(label) => {
								graph.updateDetails(renameNode.id, { name: label })
								setRenameNodeId(null)
							}}
						/>
					)}
					{eventNode && (
						<EventPickerFeature
							events={events}
							confirmLabel="Utiliser cet événement"
							onCancel={() => setEventNodeId(null)}
							onConfirm={(event) => {
								graph.updateTriggerKind(eventNode.id, { Events: [event.name] })
								setEventNodeId(null)
							}}
						/>
					)}
				</div>
			</NodeIssuesContext>
		</EditorActionsContext>
	)
}

import { Link } from '@tanstack/react-router'
import {
	applyEdgeChanges,
	applyNodeChanges,
	Background,
	Controls,
	type Edge,
	type EdgeChange,
	Handle,
	type Node,
	type NodeChange,
	type NodeProps,
	Position,
	ReactFlow,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { AlertTriangle, Loader2, Trash2, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { AuthField, Branch } from '#/hooks/use-automation'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { NodePosition } from '#/pages/automation/lib/layout'
import {
	CredentialPicker,
	type CredentialPickerOption,
} from '#/pages/automation/ui/credential-picker'
import {
	ExpressionPickerList,
	type UpstreamConnectorOption,
} from '#/pages/automation/ui/expression-picker'
import {
	type CredentialOption,
	FieldForm,
} from '#/pages/automation/ui/field-form'

/** French label for a `BranchDto` — purely cosmetic, never the value saved
 * (the wire value stays the backend's English enum; see `lib/graph.ts`'s
 * doc comment on why the canvas offers every branch generically). */
const BRANCH_LABEL: Record<Branch, string> = {
	Then: 'Alors',
	Else: 'Sinon',
	Each: 'Pour chaque élément',
	After: 'Après la boucle',
}

export interface PaletteConnector {
	kind: string
	label: string
}

export interface PaletteFamily {
	family: string
	connectors: PaletteConnector[]
}

export interface EditorNode {
	id: string
	label: string
	kindLabel: string
	hasError: boolean
}

export interface EditorEdge {
	from: string
	to: string
	branch: Branch | null
}

export interface ConnectorPanelData {
	connectorId: string
	label: string
	fields: AuthField[]
	values: Record<string, unknown>
	fieldErrors: Record<string, string>
	globalError: string | null
	authRequired: boolean
	credentialOptions: CredentialPickerOption[]
	credentialId: string | null
	signingCredentials: CredentialOption[]
}

export interface EdgePanelData {
	from: string
	to: string
	fromLabel: string
	toLabel: string
	branch: Branch | null
}

export type EditorSelection =
	| { type: 'connector'; data: ConnectorPanelData }
	| { type: 'edge'; data: EdgePanelData }

export interface WorkflowEditorUIProps {
	organizationSlug: string
	workflowId: string
	workflowName: string
	enabled: boolean
	isLoading: boolean
	error: string | null
	saveError: string | null
	isSaving: boolean
	onSave: () => void

	families: PaletteFamily[]
	onAddConnector: (kind: string) => void

	nodes: EditorNode[]
	edges: EditorEdge[]
	positions: Map<string, NodePosition>
	onConnect: (from: string, to: string) => void
	onSelectConnector: (id: string) => void
	onSelectEdge: (from: string, to: string) => void
	onDeselect: () => void

	selection: EditorSelection | null
	onFieldChange: (
		connectorId: string,
		fieldName: string,
		value: unknown,
	) => void
	onCredentialChange: (connectorId: string, credentialId: string | null) => void
	onRemoveConnector: (connectorId: string) => void
	onCreateCredential: (connectorId: string) => void
	onBranchChange: (from: string, to: string, branch: Branch | null) => void
	onRemoveEdge: (from: string, to: string) => void

	expressionTargetField: string | null
	upstreamForExpression: UpstreamConnectorOption[]
	onInsertExpression: (fieldName: string) => void
	onInsertExpressionValue: (expression: string) => void
	onCloseExpressionPicker: () => void
}

interface ConnectorNodeData {
	label: string
	kindLabel: string
	hasError: boolean
	[key: string]: unknown
}

type ConnectorNode = Node<ConnectorNodeData, 'connector'>
type BranchEdge = Edge<{ branch: Branch | null }>

function ConnectorNodeView({ data, selected }: NodeProps<ConnectorNode>) {
	return (
		<div
			className={cn(
				'min-w-40 rounded-lg border bg-card px-3 py-2 text-sm shadow-sm',
				selected && 'ring-2 ring-primary',
				data.hasError && 'border-destructive',
			)}
		>
			<Handle type="target" position={Position.Left} />
			<div className="flex items-center gap-1.5">
				{data.hasError ? (
					<AlertTriangle className="size-3.5 shrink-0 text-destructive" />
				) : null}
				<p className="truncate font-medium">{data.label}</p>
			</div>
			<p className="truncate text-xs text-muted-foreground">{data.kindLabel}</p>
			<Handle type="source" position={Position.Right} />
		</div>
	)
}

const nodeTypes = { connector: ConnectorNodeView }

export function WorkflowEditorUI(props: WorkflowEditorUIProps) {
	const {
		organizationSlug,
		workflowId,
		workflowName,
		enabled,
		isLoading,
		error,
		saveError,
		isSaving,
		onSave,
		families,
		onAddConnector,
		nodes,
		edges,
		positions,
		onConnect,
		onSelectConnector,
		onSelectEdge,
		onDeselect,
		selection,
	} = props

	const { rfNodes, rfEdges, onNodesChange, onEdgesChange } = useCanvasState(
		nodes,
		edges,
		positions,
		selection,
	)

	return (
		<PageShell>
			<PageHeader
				title={workflowName}
				eyebrow="Automatisation"
				description={
					<Link
						to={buildOrgPath(organizationSlug, '/automation/$workflowId/runs')}
						params={{ workflowId }}
						className="text-primary hover:underline"
					>
						Voir les exécutions
					</Link>
				}
				actions={
					<div className="flex items-center gap-3">
						<StatusBadge tone={enabled ? 'success' : 'neutral'}>
							{enabled ? 'Activé' : 'Désactivé'}
						</StatusBadge>
						<Button variant="ghost" asChild>
							<Link to={buildOrgPath(organizationSlug, '/automation')}>
								Retour
							</Link>
						</Button>
						<Button onClick={onSave} disabled={isSaving}>
							{isSaving ? <Loader2 className="animate-spin" /> : null}
							Enregistrer
						</Button>
					</div>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}
			{saveError ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{saveError}
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center justify-center gap-3 p-16 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement…
				</div>
			) : (
				<div className="flex gap-4">
					<Palette families={families} onAddConnector={onAddConnector} />

					<div className="h-[640px] flex-1 overflow-hidden rounded-xl border">
						<ReactFlow
							nodes={rfNodes}
							edges={rfEdges}
							onNodesChange={onNodesChange}
							onEdgesChange={onEdgesChange}
							onConnect={(connection) => {
								if (connection.source && connection.target) {
									onConnect(connection.source, connection.target)
								}
							}}
							onNodeClick={(_, node) => onSelectConnector(node.id)}
							onEdgeClick={(_, edge) => onSelectEdge(edge.source, edge.target)}
							onPaneClick={onDeselect}
							nodeTypes={nodeTypes}
							fitView
						>
							<Background />
							<Controls />
						</ReactFlow>
					</div>

					<div className="w-80 shrink-0">
						{selection === null ? (
							<SectionCard>
								<div className="p-5 text-sm text-muted-foreground">
									Sélectionnez un connecteur ou une connexion pour la
									configurer.
								</div>
							</SectionCard>
						) : selection.type === 'connector' ? (
							<ConnectorPanel {...props} data={selection.data} />
						) : (
							<EdgePanel {...props} data={selection.data} />
						)}
					</div>
				</div>
			)}
		</PageShell>
	)
}

function Palette({
	families,
	onAddConnector,
}: {
	families: PaletteFamily[]
	onAddConnector: (kind: string) => void
}) {
	return (
		<SectionCard className="h-[640px] w-56 shrink-0 overflow-y-auto">
			<SectionHeader title="Connecteurs" />
			<div className="flex flex-col gap-3 p-3">
				{families.map((group) => (
					<div key={group.family} className="flex flex-col gap-1">
						<p className="px-2 text-xs font-semibold uppercase text-muted-foreground">
							{group.family}
						</p>
						{group.connectors.map((connector) => (
							<button
								key={connector.kind}
								type="button"
								className="rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted"
								onClick={() => onAddConnector(connector.kind)}
							>
								{connector.label}
							</button>
						))}
					</div>
				))}
			</div>
		</SectionCard>
	)
}

function ConnectorPanel(
	props: WorkflowEditorUIProps & { data: ConnectorPanelData },
) {
	const { data } = props
	const targetLabel = data.fields.find(
		(field) => field.name === props.expressionTargetField,
	)?.label

	return (
		<SectionCard>
			<SectionHeader
				title={data.label}
				actions={
					<Button
						variant="ghost"
						size="icon-sm"
						title="Supprimer le connecteur"
						onClick={() => props.onRemoveConnector(data.connectorId)}
					>
						<Trash2 />
						<span className="sr-only">Supprimer</span>
					</Button>
				}
			/>
			<div className="flex flex-col gap-4 p-4">
				{data.globalError ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
						{data.globalError}
					</p>
				) : null}

				{data.authRequired ? (
					<div className="flex flex-col gap-2">
						<Label>Identification</Label>
						<CredentialPicker
							options={data.credentialOptions}
							value={data.credentialId}
							onChange={(credentialId) =>
								props.onCredentialChange(data.connectorId, credentialId)
							}
							onCreateNew={() => props.onCreateCredential(data.connectorId)}
						/>
					</div>
				) : null}

				<FieldForm
					fields={data.fields}
					values={data.values}
					errors={data.fieldErrors}
					onChange={(fieldName, value) =>
						props.onFieldChange(data.connectorId, fieldName, value)
					}
					onInsertExpression={props.onInsertExpression}
					signingCredentials={data.signingCredentials}
				/>

				{props.expressionTargetField ? (
					<div className="flex flex-col gap-2 rounded-lg border p-2">
						<div className="flex items-center justify-between px-1">
							<p className="text-xs font-medium">
								Insérer dans « {targetLabel ?? props.expressionTargetField} »
							</p>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={props.onCloseExpressionPicker}
							>
								<X />
								<span className="sr-only">Fermer</span>
							</Button>
						</div>
						<ExpressionPickerList
							upstream={props.upstreamForExpression}
							onInsert={props.onInsertExpressionValue}
						/>
					</div>
				) : null}
			</div>
		</SectionCard>
	)
}

function EdgePanel(props: WorkflowEditorUIProps & { data: EdgePanelData }) {
	const { data } = props
	return (
		<SectionCard>
			<SectionHeader
				title="Connexion"
				description={`${data.fromLabel} → ${data.toLabel}`}
				actions={
					<Button
						variant="ghost"
						size="icon-sm"
						title="Supprimer la connexion"
						onClick={() => props.onRemoveEdge(data.from, data.to)}
					>
						<Trash2 />
						<span className="sr-only">Supprimer</span>
					</Button>
				}
			/>
			<div className="flex flex-col gap-2 p-4">
				<Label htmlFor="edge-branch">Branche</Label>
				<Select
					value={data.branch ?? 'none'}
					onValueChange={(next) =>
						props.onBranchChange(
							data.from,
							data.to,
							next === 'none' ? null : (next as Branch),
						)
					}
				>
					<SelectTrigger id="edge-branch" className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="none">Aucune</SelectItem>
						{(Object.keys(BRANCH_LABEL) as Branch[]).map((branch) => (
							<SelectItem key={branch} value={branch}>
								{BRANCH_LABEL[branch]}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		</SectionCard>
	)
}

/**
 * Bridges the feature's canonical `nodes`/`edges` props into React Flow's
 * own node/edge state. Position is the one thing React Flow must own
 * itself — dragging a node has to feel instant, and the wire model has no
 * `x`/`y` to round-trip through the feature anyway (see `lib/layout.ts`).
 * Everything else (label, error flag, branch, selection) re-syncs from
 * props on every change; only a node's position, once dragged, survives
 * across syncs, via `draggedPositions`.
 */
function useCanvasState(
	nodes: EditorNode[],
	edges: EditorEdge[],
	positions: Map<string, NodePosition>,
	selection: EditorSelection | null,
) {
	const draggedPositions = useRef(new Map<string, NodePosition>())
	const [rfNodes, setRfNodes] = useState<ConnectorNode[]>([])
	const [rfEdges, setRfEdges] = useState<BranchEdge[]>([])

	// `positions` is recomputed fresh every render from `nodes`/`edges` (a
	// pure function, see `lib/layout.ts`) — depending on `nodes` here rather
	// than on `positions` itself is what keeps a drag from being reset by an
	// unrelated re-render that happens to recompute the same layout.
	// biome-ignore lint/correctness/useExhaustiveDependencies: see above
	useEffect(() => {
		setRfNodes(
			nodes.map((node) => ({
				id: node.id,
				type: 'connector',
				position: draggedPositions.current.get(node.id) ??
					positions.get(node.id) ?? { x: 0, y: 0 },
				selected:
					selection?.type === 'connector' &&
					selection.data.connectorId === node.id,
				data: {
					label: node.label,
					kindLabel: node.kindLabel,
					hasError: node.hasError,
				},
			})),
		)
	}, [nodes, selection])

	useEffect(() => {
		setRfEdges(
			edges.map((edge) => ({
				id: `${edge.from}->${edge.to}`,
				source: edge.from,
				target: edge.to,
				label: edge.branch ? BRANCH_LABEL[edge.branch] : undefined,
				selected:
					selection?.type === 'edge' &&
					selection.data.from === edge.from &&
					selection.data.to === edge.to,
				data: { branch: edge.branch },
			})),
		)
	}, [edges, selection])

	function onNodesChange(changes: NodeChange<ConnectorNode>[]) {
		for (const change of changes) {
			if (change.type === 'position' && change.position) {
				draggedPositions.current.set(change.id, change.position)
			}
		}
		setRfNodes((current) => applyNodeChanges(changes, current))
	}

	function onEdgesChange(changes: EdgeChange<BranchEdge>[]) {
		setRfEdges((current) => applyEdgeChanges(changes, current))
	}

	return { rfNodes, rfEdges, onNodesChange, onEdgesChange }
}

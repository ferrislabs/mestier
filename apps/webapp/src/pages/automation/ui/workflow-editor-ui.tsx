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
import {
	AlertTriangle,
	Copy,
	Loader2,
	MoreHorizontal,
	Plus,
	Trash2,
	X,
	Zap,
} from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverAnchor,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
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
import { badgeColorFor } from '#/pages/automation/lib/connector-badge'
import { COLUMN_WIDTH, type NodePosition } from '#/pages/automation/lib/layout'
import { ConnectorSearchList } from '#/pages/automation/ui/connector-search-list'
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

/** The canvas's entry-point pseudo-node — never sent to the backend (see
 * `lib/graph.ts`'s doc comment on `rootConnectorIds`: the graph itself has
 * no trigger node, a connector's in-degree of zero already means "a run
 * starts here"). Reserved so `onNodeClick`/`onNodeContextMenu` can
 * recognize and ignore it — it is not a `PlacedConnector` and has nothing
 * a side panel could show. */
const START_NODE_ID = '__start__'

/** French label for a `BranchDto` — purely cosmetic, never the value saved
 * (the wire value stays the backend's English enum; see `lib/graph.ts`'s
 * doc comment on why the canvas offers every branch generically). */
const BRANCH_LABEL: Record<Branch, string> = {
	Then: 'Alors',
	Else: 'Sinon',
	Each: 'Pour chaque élément',
	After: 'Après la boucle',
}

/** A branch label renders as a pill sitting directly on the edge — same
 * mint treatment as `StatusBadge`'s `success` tone (`--success`/
 * `--success-soft`), just expressed as raw SVG paint since React Flow
 * draws edge labels with `<text>`/`<rect>`, not HTML/Tailwind. */
const BRANCH_PILL_STYLE = {
	labelBgPadding: [8, 4] as [number, number],
	labelBgBorderRadius: 6,
	labelStyle: { fill: 'var(--success)', fontWeight: 600, fontSize: 11 },
	labelBgStyle: { fill: 'var(--success-soft)' },
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
	/** The connector's catalogue `family` — drives the node's badge color
	 * and, on the palette, groups it with its siblings. `''` for a placed
	 * connector whose kind was retired from the catalogue. */
	family: string
	hasError: boolean
	/** Up to two "Label: value" lines from the connector's own config — see
	 * `lib/graph.ts`'s `previewLinesFor`. Never a secret field's value. */
	previewLines: string[]
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

/** The edge's config lives entirely in the right-click context menu now
 * (branch quick-pick + delete) — nothing renders from `selection` for an
 * edge beyond the `selected` highlight ring, so unlike a connector it
 * carries no further data. */
export type EditorSelection =
	| { type: 'connector'; data: ConnectorPanelData }
	| { type: 'edge'; from: string; to: string }

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
	/** The very first connectors in the graph — nothing points into them,
	 * so the canvas draws the Start pseudo-node's edges to exactly these. */
	rootConnectorIds: string[]

	nodes: EditorNode[]
	edges: EditorEdge[]
	positions: Map<string, NodePosition>
	onConnect: (from: string, to: string) => void
	/** From a node's own "+" button, a right-click "Ajouter après", or a
	 * manual drag from its handle — every way of extending the graph past
	 * an existing connector funnels through this one call. */
	onAddConnectorFrom: (fromId: string, kind: string) => void
	onDuplicateConnector: (connectorId: string) => void
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
	family: string
	hasError: boolean
	previewLines: string[]
	families: PaletteFamily[]
	onAddFrom: (kind: string) => void
	onDuplicate: () => void
	onRemove: () => void
	/** Non-null exactly when this node is the selected connector — the
	 * config `Popover`'s open state is driven directly off this, not off
	 * separate click handling (see `ConnectorNodeView`). */
	panel: ConnectorPanelData | null
	onFieldChange: (fieldName: string, value: unknown) => void
	onCredentialChange: (credentialId: string | null) => void
	onCreateCredential: () => void
	expressionTargetField: string | null
	upstreamForExpression: UpstreamConnectorOption[]
	onInsertExpression: (fieldName: string) => void
	onInsertExpressionValue: (expression: string) => void
	onCloseExpressionPicker: () => void
	onDeselect: () => void
	[key: string]: unknown
}

type ConnectorNode = Node<ConnectorNodeData, 'connector'>
type StartNode = Node<Record<string, never>, 'start'>
type BranchEdge = Edge<{ branch: Branch | null }>

function ConnectorNodeView({ data, selected }: NodeProps<ConnectorNode>) {
	return (
		// Open state is driven entirely by `data.panel` (non-null exactly
		// when the feature considers this connector selected), never by this
		// Popover's own interaction-outside logic: Radix's default dismiss-
		// on-outside-click would otherwise fire for a click on *any* other
		// UI — "Enregistrer", the palette, another node — deselecting this
		// connector as a side effect of an unrelated click, so
		// `onInteractOutside` is disabled below. Clicking the card sets
		// selection via React Flow's `onNodeClick`, and that alone is what
		// opens it; the pane's own click handler and Escape are the only
		// two things that close it, both explicit.
		<Popover open={data.panel !== null}>
			<PopoverAnchor asChild>
				<div
					className={cn(
						'relative min-w-56 rounded-xl border bg-card px-3 py-2.5 text-sm shadow-sm transition',
						selected && 'ring-2 ring-primary',
						data.hasError && 'border-destructive',
					)}
				>
					<Handle type="target" position={Position.Left} />
					<div className="flex items-center gap-2">
						<span
							className={cn(
								'flex size-6 shrink-0 items-center justify-center rounded-md text-xs font-semibold',
								badgeColorFor(data.family || data.kindLabel),
							)}
						>
							{(data.family || data.kindLabel).charAt(0).toUpperCase()}
						</span>
						<p className="truncate font-medium">{data.label}</p>
						{data.hasError ? (
							<AlertTriangle className="size-3.5 shrink-0 text-destructive" />
						) : null}
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<button
									type="button"
									className="nodrag ml-auto flex size-6 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground"
								>
									<MoreHorizontal className="size-4" />
									<span className="sr-only">Actions</span>
								</button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end">
								<DropdownMenuItem onSelect={data.onDuplicate}>
									<Copy />
									Dupliquer
								</DropdownMenuItem>
								<DropdownMenuSeparator />
								<DropdownMenuItem
									variant="destructive"
									onSelect={data.onRemove}
								>
									<Trash2 />
									Supprimer
								</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</div>
					<p className="mt-1 truncate pl-8 text-xs text-muted-foreground">
						{data.kindLabel}
					</p>
					{data.previewLines.length > 0 ? (
						<div className="mt-2 flex flex-col gap-0.5 rounded-md bg-muted/60 px-2 py-1.5">
							{data.previewLines.map((line) => (
								<p
									key={line}
									className="truncate font-mono text-[11px] text-muted-foreground"
								>
									{line}
								</p>
							))}
						</div>
					) : null}
					<Handle type="source" position={Position.Right} />
					<Popover>
						<PopoverTrigger asChild>
							<button
								type="button"
								title="Ajouter un connecteur après celui-ci"
								className="nodrag absolute top-1/2 -right-9 flex size-6 -translate-y-1/2 items-center justify-center rounded-full border bg-background text-muted-foreground shadow-sm transition hover:bg-muted hover:text-foreground"
							>
								<Plus className="size-3.5" />
								<span className="sr-only">Ajouter après</span>
							</button>
						</PopoverTrigger>
						<PopoverContent className="w-auto p-0" align="start" side="right">
							<ConnectorSearchList
								families={data.families}
								onSelect={data.onAddFrom}
							/>
						</PopoverContent>
					</Popover>
				</div>
			</PopoverAnchor>
			<PopoverContent
				className="w-96 p-0"
				side="right"
				align="start"
				onInteractOutside={(event) => event.preventDefault()}
				onEscapeKeyDown={(event) => {
					event.preventDefault()
					data.onDeselect()
				}}
			>
				{data.panel ? (
					<ConnectorConfigContent
						panel={data.panel}
						onFieldChange={data.onFieldChange}
						onCredentialChange={data.onCredentialChange}
						onCreateCredential={data.onCreateCredential}
						onRemoveConnector={data.onRemove}
						expressionTargetField={data.expressionTargetField}
						upstreamForExpression={data.upstreamForExpression}
						onInsertExpression={data.onInsertExpression}
						onInsertExpressionValue={data.onInsertExpressionValue}
						onCloseExpressionPicker={data.onCloseExpressionPicker}
					/>
				) : null}
			</PopoverContent>
		</Popover>
	)
}

function StartNodeView() {
	return (
		<div className="flex items-center gap-2 rounded-full border bg-success-soft px-4 py-2.5 text-sm font-semibold text-success shadow-sm">
			<Zap className="size-4" />
			Départ
			<Handle type="source" position={Position.Right} isConnectable={false} />
		</div>
	)
}

const nodeTypes = { connector: ConnectorNodeView, start: StartNodeView }

type ContextMenuState =
	| { type: 'node'; id: string; x: number; y: number }
	| {
			type: 'edge'
			from: string
			to: string
			branch: Branch | null
			x: number
			y: number
	  }
	| { type: 'pane'; x: number; y: number }

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
		rootConnectorIds,
		nodes,
		edges,
		positions,
		onConnect,
		onAddConnectorFrom,
		onDuplicateConnector,
		onSelectConnector,
		onSelectEdge,
		onDeselect,
		onFieldChange,
		onCredentialChange,
		onRemoveConnector,
		onCreateCredential,
		onRemoveEdge,
		onBranchChange,
		selection,
		expressionTargetField,
		upstreamForExpression,
		onInsertExpression,
		onInsertExpressionValue,
		onCloseExpressionPicker,
	} = props

	const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)

	const { rfNodes, rfEdges, onNodesChange, onEdgesChange } = useCanvasState({
		nodes,
		edges,
		positions,
		selection,
		families,
		rootConnectorIds,
		onAddConnectorFrom,
		onDuplicateConnector,
		onRemoveConnector,
		onFieldChange,
		onCredentialChange,
		onCreateCredential,
		onDeselect,
		expressionTargetField,
		upstreamForExpression,
		onInsertExpression,
		onInsertExpressionValue,
		onCloseExpressionPicker,
	})

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
				<div className="flex gap-6">
					<Palette families={families} onAddConnector={onAddConnector} />

					<div className="h-[calc(100vh-19rem)] min-h-[560px] flex-1 overflow-hidden rounded-xl border bg-muted/10">
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
							onNodeClick={(_, node) => {
								if (node.id === START_NODE_ID) return
								onSelectConnector(node.id)
							}}
							onEdgeClick={(_, edge) => onSelectEdge(edge.source, edge.target)}
							onPaneClick={() => {
								onDeselect()
								setContextMenu(null)
							}}
							onNodeContextMenu={(event, node) => {
								if (node.id === START_NODE_ID) return
								event.preventDefault()
								setContextMenu({
									type: 'node',
									id: node.id,
									x: event.clientX,
									y: event.clientY,
								})
							}}
							onEdgeContextMenu={(event, edge) => {
								event.preventDefault()
								const branchEdge = edge as BranchEdge
								setContextMenu({
									type: 'edge',
									from: branchEdge.source,
									to: branchEdge.target,
									branch: branchEdge.data?.branch ?? null,
									x: event.clientX,
									y: event.clientY,
								})
							}}
							onPaneContextMenu={(event) => {
								event.preventDefault()
								setContextMenu({
									type: 'pane',
									x: (event as MouseEvent).clientX,
									y: (event as MouseEvent).clientY,
								})
							}}
							onMoveStart={() => setContextMenu(null)}
							nodeTypes={nodeTypes}
							defaultEdgeOptions={{ deletable: true }}
							fitView
						>
							<Background gap={24} />
							<Controls />
						</ReactFlow>
					</div>
				</div>
			)}

			<ContextMenu
				menu={contextMenu}
				families={families}
				onClose={() => setContextMenu(null)}
				onAddConnector={onAddConnector}
				onDuplicateConnector={onDuplicateConnector}
				onRemoveConnector={onRemoveConnector}
				onRemoveEdge={onRemoveEdge}
				onBranchChange={onBranchChange}
			/>
		</PageShell>
	)
}

function ContextMenu({
	menu,
	families,
	onClose,
	onAddConnector,
	onDuplicateConnector,
	onRemoveConnector,
	onRemoveEdge,
	onBranchChange,
}: {
	menu: ContextMenuState | null
	families: PaletteFamily[]
	onClose: () => void
	onAddConnector: (kind: string) => void
	onDuplicateConnector: (connectorId: string) => void
	onRemoveConnector: (connectorId: string) => void
	onRemoveEdge: (from: string, to: string) => void
	onBranchChange: (from: string, to: string, branch: Branch | null) => void
}) {
	if (menu === null) return null

	// A zero-size anchor fixed at the click's screen position — the menu's
	// `open` state is driven entirely by `menu !== null`, never by clicking
	// this anchor itself, so it can appear anywhere the cursor right-clicked
	// rather than only next to a visible trigger button.
	const anchor = (
		<span
			style={{ position: 'fixed', left: menu.x, top: menu.y }}
			className="pointer-events-none"
		/>
	)

	if (menu.type === 'pane') {
		return (
			<Popover open onOpenChange={(open) => !open && onClose()}>
				<PopoverAnchor asChild>{anchor}</PopoverAnchor>
				<PopoverContent className="w-auto p-0" align="start">
					<ConnectorSearchList
						families={families}
						onSelect={(kind) => {
							onAddConnector(kind)
							onClose()
						}}
					/>
				</PopoverContent>
			</Popover>
		)
	}

	return (
		<DropdownMenu open onOpenChange={(open) => !open && onClose()}>
			<DropdownMenuTrigger asChild>{anchor}</DropdownMenuTrigger>
			<DropdownMenuContent align="start">
				{menu.type === 'node' ? (
					<>
						<DropdownMenuItem
							onSelect={() => {
								onDuplicateConnector(menu.id)
								onClose()
							}}
						>
							<Copy />
							Dupliquer
						</DropdownMenuItem>
						<DropdownMenuSeparator />
						<DropdownMenuItem
							variant="destructive"
							onSelect={() => {
								onRemoveConnector(menu.id)
								onClose()
							}}
						>
							<Trash2 />
							Supprimer
						</DropdownMenuItem>
					</>
				) : (
					<>
						<DropdownMenuItem
							onSelect={() => {
								onBranchChange(menu.from, menu.to, null)
								onClose()
							}}
						>
							{menu.branch === null ? '✓ ' : null}Aucune branche
						</DropdownMenuItem>
						{(Object.keys(BRANCH_LABEL) as Branch[]).map((branch) => (
							<DropdownMenuItem
								key={branch}
								onSelect={() => {
									onBranchChange(menu.from, menu.to, branch)
									onClose()
								}}
							>
								{menu.branch === branch ? '✓ ' : null}
								{BRANCH_LABEL[branch]}
							</DropdownMenuItem>
						))}
						<DropdownMenuSeparator />
						<DropdownMenuItem
							variant="destructive"
							onSelect={() => {
								onRemoveEdge(menu.from, menu.to)
								onClose()
							}}
						>
							<Trash2 />
							Supprimer la connexion
						</DropdownMenuItem>
					</>
				)}
			</DropdownMenuContent>
		</DropdownMenu>
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
		<SectionCard className="h-[calc(100vh-19rem)] min-h-[560px] w-64 shrink-0 overflow-y-auto">
			<SectionHeader
				title="Connecteurs"
				description="Cliquez pour ajouter, ou faites glisser depuis le « + » d’un connecteur existant."
			/>
			<div className="flex flex-col gap-4 p-4">
				{families.map((group) => (
					<div key={group.family} className="flex flex-col gap-1">
						<p className="px-2 text-xs font-semibold uppercase text-muted-foreground">
							{group.family}
						</p>
						{group.connectors.map((connector) => (
							<button
								key={connector.kind}
								type="button"
								className="flex items-center gap-2 rounded-md px-2 py-2 text-left text-sm hover:bg-muted"
								onClick={() => onAddConnector(connector.kind)}
							>
								<span
									className={cn(
										'flex size-6 shrink-0 items-center justify-center rounded-md text-[10px] font-semibold',
										badgeColorFor(group.family),
									)}
								>
									{group.family.charAt(0).toUpperCase()}
								</span>
								<span className="truncate">{connector.label}</span>
							</button>
						))}
					</div>
				))}
			</div>
		</SectionCard>
	)
}

export interface ConnectorConfigContentProps {
	panel: ConnectorPanelData
	onFieldChange: (fieldName: string, value: unknown) => void
	onCredentialChange: (credentialId: string | null) => void
	onCreateCredential: () => void
	onRemoveConnector: () => void
	expressionTargetField: string | null
	upstreamForExpression: UpstreamConnectorOption[]
	onInsertExpression: (fieldName: string) => void
	onInsertExpressionValue: (expression: string) => void
	onCloseExpressionPicker: () => void
}

/** A connector's config, edited from a `Popover` anchored to its own card —
 * there is no persistent side panel for it anymore (see this file's
 * `useCanvasState` doc comment): the canvas keeps the full width, and a
 * connector's behavior is a dropdown away, the same way its "..." menu
 * already is. */
export function ConnectorConfigContent({
	panel,
	onFieldChange,
	onCredentialChange,
	onCreateCredential,
	onRemoveConnector,
	expressionTargetField,
	upstreamForExpression,
	onInsertExpression,
	onInsertExpressionValue,
	onCloseExpressionPicker,
}: ConnectorConfigContentProps) {
	const targetLabel = panel.fields.find(
		(field) => field.name === expressionTargetField,
	)?.label

	return (
		<div className="flex flex-col">
			<div className="flex items-center justify-between gap-4 border-b px-4 py-3">
				<p className="font-medium">{panel.label}</p>
				<Button
					variant="ghost"
					size="icon-sm"
					title="Supprimer le connecteur"
					onClick={onRemoveConnector}
				>
					<Trash2 />
					<span className="sr-only">Supprimer</span>
				</Button>
			</div>
			<div className="flex max-h-[70vh] flex-col gap-5 overflow-y-auto p-4">
				{panel.globalError ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
						{panel.globalError}
					</p>
				) : null}

				{panel.authRequired ? (
					<div className="flex flex-col gap-2">
						<Label>Identification</Label>
						<CredentialPicker
							options={panel.credentialOptions}
							value={panel.credentialId}
							onChange={onCredentialChange}
							onCreateNew={onCreateCredential}
						/>
					</div>
				) : null}

				<FieldForm
					fields={panel.fields}
					values={panel.values}
					errors={panel.fieldErrors}
					onChange={onFieldChange}
					onInsertExpression={onInsertExpression}
					signingCredentials={panel.signingCredentials}
				/>

				{expressionTargetField ? (
					<div className="flex flex-col gap-2 rounded-lg border p-2">
						<div className="flex items-center justify-between px-1">
							<p className="text-xs font-medium">
								Insérer dans « {targetLabel ?? expressionTargetField} »
							</p>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={onCloseExpressionPicker}
							>
								<X />
								<span className="sr-only">Fermer</span>
							</Button>
						</div>
						<ExpressionPickerList
							upstream={upstreamForExpression}
							onInsert={onInsertExpressionValue}
						/>
					</div>
				) : null}
			</div>
		</div>
	)
}

/**
 * Bridges the feature's canonical `nodes`/`edges` props into React Flow's
 * own node/edge state, and prepends the visual-only Start node + its
 * synthetic edges to every root connector. Position is the one thing
 * React Flow must own itself — dragging a node has to feel instant, and
 * the wire model has no `x`/`y` to round-trip through the feature anyway
 * (see `lib/layout.ts`). Everything else (label, error flag, branch,
 * selection) re-syncs from props on every change; only a node's position,
 * once dragged, survives across syncs, via `draggedPositions`.
 */
function useCanvasState({
	nodes,
	edges,
	positions,
	selection,
	families,
	rootConnectorIds,
	onAddConnectorFrom,
	onDuplicateConnector,
	onRemoveConnector,
	onFieldChange,
	onCredentialChange,
	onCreateCredential,
	onDeselect,
	expressionTargetField,
	upstreamForExpression,
	onInsertExpression,
	onInsertExpressionValue,
	onCloseExpressionPicker,
}: {
	nodes: EditorNode[]
	edges: EditorEdge[]
	positions: Map<string, NodePosition>
	selection: EditorSelection | null
	families: PaletteFamily[]
	rootConnectorIds: string[]
	onAddConnectorFrom: (fromId: string, kind: string) => void
	onDuplicateConnector: (connectorId: string) => void
	onRemoveConnector: (connectorId: string) => void
	onFieldChange: (
		connectorId: string,
		fieldName: string,
		value: unknown,
	) => void
	onCredentialChange: (connectorId: string, credentialId: string | null) => void
	onCreateCredential: (connectorId: string) => void
	onDeselect: () => void
	expressionTargetField: string | null
	upstreamForExpression: UpstreamConnectorOption[]
	onInsertExpression: (fieldName: string) => void
	onInsertExpressionValue: (expression: string) => void
	onCloseExpressionPicker: () => void
}) {
	const draggedPositions = useRef(new Map<string, NodePosition>())
	const [rfNodes, setRfNodes] = useState<Array<ConnectorNode | StartNode>>([])
	const [rfEdges, setRfEdges] = useState<BranchEdge[]>([])

	// `positions` is recomputed fresh every render from `nodes`/`edges` (a
	// pure function, see `lib/layout.ts`) — depending on `nodes` here rather
	// than on `positions` itself is what keeps a drag from being reset by an
	// unrelated re-render that happens to recompute the same layout.
	// biome-ignore lint/correctness/useExhaustiveDependencies: see above
	useEffect(() => {
		const rootYs = rootConnectorIds
			.map((id) => positions.get(id)?.y)
			.filter((y): y is number => y !== undefined)
		const startY =
			rootYs.length > 0
				? rootYs.reduce((sum, y) => sum + y, 0) / rootYs.length
				: 0
		const startWidth = 110
		const startHeight = 44
		const startNode: StartNode = {
			id: START_NODE_ID,
			type: 'start',
			position:
				draggedPositions.current.get(START_NODE_ID) ??
				({ x: -COLUMN_WIDTH, y: startY } as NodePosition),
			draggable: true,
			selectable: false,
			// A rough guess, immediately overwritten by React Flow's own
			// measurement once it observes the real element — but until then
			// it's what keeps the node from rendering `visibility: hidden`
			// for one frame (`nodeHasDimensions()` accepts an initial guess),
			// and `handles` is what lets an edge attach to it without waiting
			// on a real handle measurement either.
			initialWidth: startWidth,
			initialHeight: startHeight,
			handles: [
				{
					type: 'source',
					position: Position.Right,
					x: startWidth,
					y: startHeight / 2,
				},
			],
			data: {},
		}

		setRfNodes([
			startNode,
			...nodes.map((node) => {
				const width = 240
				const height = node.previewLines.length > 0 ? 130 : 90
				return {
					id: node.id,
					type: 'connector' as const,
					position: draggedPositions.current.get(node.id) ??
						positions.get(node.id) ?? { x: 0, y: 0 },
					selected:
						selection?.type === 'connector' &&
						selection.data.connectorId === node.id,
					// Same rough-guess purpose as the Start node's above.
					initialWidth: width,
					initialHeight: height,
					handles: [
						{
							type: 'target' as const,
							position: Position.Left,
							x: 0,
							y: height / 2,
						},
						{
							type: 'source' as const,
							position: Position.Right,
							x: width,
							y: height / 2,
						},
					],
					data: {
						label: node.label,
						kindLabel: node.kindLabel,
						family: node.family,
						hasError: node.hasError,
						previewLines: node.previewLines,
						families,
						onAddFrom: (kind: string) => onAddConnectorFrom(node.id, kind),
						onDuplicate: () => onDuplicateConnector(node.id),
						onRemove: () => onRemoveConnector(node.id),
						panel:
							selection?.type === 'connector' &&
							selection.data.connectorId === node.id
								? selection.data
								: null,
						onFieldChange: (fieldName: string, value: unknown) =>
							onFieldChange(node.id, fieldName, value),
						onCredentialChange: (credentialId: string | null) =>
							onCredentialChange(node.id, credentialId),
						onCreateCredential: () => onCreateCredential(node.id),
						onDeselect,
						expressionTargetField:
							selection?.type === 'connector' &&
							selection.data.connectorId === node.id
								? expressionTargetField
								: null,
						upstreamForExpression,
						onInsertExpression,
						onInsertExpressionValue,
						onCloseExpressionPicker,
					},
				}
			}),
		])
	}, [
		nodes,
		selection,
		rootConnectorIds,
		expressionTargetField,
		upstreamForExpression,
	])

	useEffect(() => {
		const startEdges: BranchEdge[] = rootConnectorIds.map((id) => ({
			id: `${START_NODE_ID}->${id}`,
			source: START_NODE_ID,
			target: id,
			deletable: false,
			data: { branch: null },
		}))

		setRfEdges([
			...startEdges,
			...edges.map((edge) => ({
				id: `${edge.from}->${edge.to}`,
				source: edge.from,
				target: edge.to,
				label: edge.branch ? BRANCH_LABEL[edge.branch] : undefined,
				...(edge.branch ? BRANCH_PILL_STYLE : null),
				selected:
					selection?.type === 'edge' &&
					selection.from === edge.from &&
					selection.to === edge.to,
				data: { branch: edge.branch },
			})),
		])
	}, [edges, selection, rootConnectorIds])

	function onNodesChange(changes: NodeChange<ConnectorNode | StartNode>[]) {
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

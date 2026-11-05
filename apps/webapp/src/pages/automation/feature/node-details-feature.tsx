import { Link } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import {
	ConnectorParametersFeature,
	type ConnectorParametersFeatureProps,
} from '#/pages/automation/feature/connector-parameters-feature'
import { EventPickerFeature } from '#/pages/automation/feature/event-picker-feature'
import { useCollections } from '#/pages/automation/feature/use-collections'
import { identificationsPath } from '#/pages/automation/lib/automation-paths'
import { branchLabel } from '#/pages/automation/lib/branch-labels'
import { collectionCredentialHint } from '#/pages/automation/lib/connector-field-copy'
import type {
	EditorEdge,
	EditorNode,
	NodeMeta,
} from '#/pages/automation/lib/editor-nodes'
import { type Neighbour, neighboursOf } from '#/pages/automation/lib/graph'
import { appearanceOf } from '#/pages/automation/lib/node-appearance'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import {
	type NodeInputView,
	nodeDescription,
	outputViewFromPayload,
	outputViewFromStep,
} from '#/pages/automation/lib/node-view'
import {
	operationSuccessExample,
	resolveOperation,
} from '#/pages/automation/lib/operation-search'
import { labelConflict } from '#/pages/automation/lib/references'
import {
	findEvent,
	triggerEventNames,
} from '#/pages/automation/lib/trigger-events'
import {
	evaluateContextFor,
	type UpstreamSources,
	upstreamEntries,
} from '#/pages/automation/lib/upstream-entries'
import { EventInputPanel } from '#/pages/automation/ui/event-input-panel'
import { InlineEditField } from '#/pages/automation/ui/inline-edit-field'
import type { NdvNavItem } from '#/pages/automation/ui/ndv-node-nav'
import { NodeDetailsDialog } from '#/pages/automation/ui/node-details-dialog'
import { NodeInputSection } from '#/pages/automation/ui/node-input-section'
import { NodeOutputPanel } from '#/pages/automation/ui/node-output-panel'
import {
	NodeParametersPanel,
	OperationField,
} from '#/pages/automation/ui/node-parameters-panel'
import { OperationOutputPanel } from '#/pages/automation/ui/operation-output-panel'
import { RunStatusLabel } from '#/pages/automation/ui/run-status-label'
import { TriggerParameters } from '#/pages/automation/ui/trigger-parameters'
import type { TestRun } from '#/pages/automation/ui/variables-panel'

export interface NodeDetailsFeatureProps {
	node: EditorNode
	nodes: readonly EditorNode[]
	edges: readonly EditorEdge[]
	organizationSlug: string
	sources: UpstreamSources
	authSchemes: Schemas.AuthSchemeResponse[]
	credentials: Schemas.CredentialResponse[]
	issues: readonly NodeIssue[]
	testRun: TestRun
	onUpdateConnector: (
		patch: Partial<
			Pick<Schemas.PlacedConnectorDto, 'config' | 'credential_id'>
		>,
	) => void
	onUpdateTriggerKind: (kind: Schemas.TriggerKindDto) => void
	onUpdateMeta: (patch: NodeMeta) => void
	onCreateCredential: ConnectorParametersFeatureProps['onCreateCredential']
	onEvaluateExpression: ConnectorParametersFeatureProps['onEvaluateExpression']
	onClose: () => void
	onNavigate: (nodeId: string) => void
}

function byVerticalPosition(
	nodes: ReadonlyMap<string, EditorNode>,
): (a: Neighbour, b: Neighbour) => number {
	return (a, b) =>
		(nodes.get(a.id)?.position.y ?? 0) - (nodes.get(b.id)?.position.y ?? 0)
}

export function NodeDetailsFeature({
	node,
	nodes,
	edges,
	organizationSlug,
	sources,
	authSchemes,
	credentials,
	issues,
	testRun,
	onUpdateConnector,
	onUpdateTriggerKind,
	onUpdateMeta,
	onCreateCredential,
	onEvaluateExpression,
	onClose,
	onNavigate,
}: NodeDetailsFeatureProps) {
	const [picking, setPicking] = useState<{ replacing: string | null } | null>(
		null,
	)
	const { connector, trigger } = node.data
	const collections = useCollections().data
	const operationRef = node.data.meta.operation
	const operation = operationRef
		? resolveOperation(operationRef, collections)
		: null
	const successExample = operation ? operationSuccessExample(operation) : null
	const descriptor = connector
		? sources.descriptors.get(connector.kind)
		: undefined
	const appearance = appearanceOf(node.data)
	const byId = new Map(nodes.map((candidate) => [candidate.id, candidate]))
	const entries = upstreamEntries(node.id, nodes, edges, sources)
	const neighbours = neighboursOf(node.id, edges)
	const toNavItem =
		(withBranch: boolean) =>
		(neighbour: Neighbour): NdvNavItem[] => {
			const target = byId.get(neighbour.id)
			if (!target) return []
			const look = appearanceOf(target.data)
			return [
				{
					id: target.id,
					label: target.data.label,
					icon: look.icon,
					hue: look.hue,
					branch:
						withBranch && neighbour.handle
							? branchLabel(neighbour.handle)
							: undefined,
				},
			]
		}
	const evaluateContext = useMemo(
		() => evaluateContextFor(node.id, nodes, edges, sources),
		[node.id, nodes, edges, sources],
	)
	const eventNames = trigger ? triggerEventNames(trigger) : []
	const chosenEvents = eventNames.flatMap((name) => {
		const event = findEvent(sources.events, name)
		return event ? [event] : []
	})
	const currentEvent = chosenEvents[0] ?? null
	const step = connector ? sources.runSteps.get(connector.id) : undefined
	const inputFallback: NodeInputView = trigger
		? { state: 'trigger' }
		: { state: 'unconnected' }

	function applyEventChoice(event: Schemas.EventDescriptorResponse) {
		if (!trigger || trigger.kind === 'Manual' || !picking) return
		const names = trigger.kind.Events
		const next =
			picking.replacing === null
				? [...names.filter((name) => name !== event.name), event.name]
				: names.map((name) => (name === picking.replacing ? event.name : name))
		onUpdateTriggerKind({ Events: [...new Set(next)] })
		setPicking(null)
	}

	const fieldIssues = issues
	const acceptsExpressions =
		descriptor?.fields.some((field) => field.expression) ?? false

	return (
		<>
			<NodeDetailsDialog
				open
				onOpenChange={(open) => {
					if (!open) onClose()
				}}
				title={node.data.label}
				description={nodeDescription(node.data)}
				heading={
					<div
						key={`${node.id}:${node.data.label}:${node.data.subtitle}`}
						className="grid min-w-0"
					>
						<InlineEditField
							value={node.data.label}
							label="Nom du nœud"
							validate={(name) => labelConflict(name, node.id, nodes)}
							onCommit={(label) => onUpdateMeta({ label })}
							className="font-semibold text-base leading-tight"
						/>
						<InlineEditField
							value={node.data.subtitle}
							label="Description du nœud"
							placeholder="Ajouter une description"
							onCommit={(subtitle) => onUpdateMeta({ subtitle })}
							className="text-muted-foreground text-sm"
						/>
					</div>
				}
				badge={
					<span className="hidden shrink-0 rounded-sm border border-border px-1.5 py-0.5 text-muted-foreground text-xs sm:inline">
						{node.data.typeLabel}
					</span>
				}
				icon={appearance.icon}
				hue={appearance.hue}
				status={step && <RunStatusLabel status={step.status} />}
				input={
					trigger && trigger.kind !== 'Manual' ? (
						<EventInputPanel
							events={chosenEvents}
							onChooseEvent={() => setPicking({ replacing: null })}
						/>
					) : (
						<NodeInputSection
							entries={entries}
							fallback={inputFallback}
							testRun={testRun}
						/>
					)
				}
				parameters={
					<NodeParametersPanel acceptsExpressions={acceptsExpressions}>
						{operationRef && (
							<OperationField
								operation={
									operation
										? {
												method: operation.operation.method,
												path: operation.operation.path,
												collectionTitle: operation.collection.title,
												baseUrl: operation.collection.baseUrl,
											}
										: null
								}
							/>
						)}
						{connector && descriptor ? (
							<ConnectorParametersFeature
								key={node.id}
								descriptor={descriptor}
								config={connector.config}
								credentialId={connector.credential_id ?? null}
								credentials={credentials}
								authSchemes={authSchemes}
								errors={node.data.errors}
								bindingIssues={fieldIssues}
								credentialHint={
									operation
										? collectionCredentialHint(operation.collection.security)
										: undefined
								}
								evaluateContext={evaluateContext}
								onConfigChange={(config) => onUpdateConnector({ config })}
								onCredentialChange={(credentialId) =>
									onUpdateConnector({ credential_id: credentialId })
								}
								onCreateCredential={onCreateCredential}
								onEvaluateExpression={onEvaluateExpression}
							/>
						) : trigger ? (
							<>
								{node.data.errors.length > 0 && (
									<div
										role="alert"
										className="rounded-md border border-state-alert/60 bg-state-alert/10 px-3 py-2 text-sm text-state-alert"
									>
										{node.data.errors.map((error) => (
											<p key={error.message}>{error.message}</p>
										))}
									</div>
								)}
								<TriggerParameters
									trigger={trigger}
									events={sources.events}
									onChooseEvent={(replacing) => setPicking({ replacing })}
									onRemoveEvent={(name) =>
										onUpdateTriggerKind({
											Events: eventNames.filter(
												(candidate) => candidate !== name,
											),
										})
									}
								/>
							</>
						) : null}
					</NodeParametersPanel>
				}
				previous={neighbours.previous
					.sort(byVerticalPosition(byId))
					.flatMap(toNavItem(false))}
				next={neighbours.next
					.sort(byVerticalPosition(byId))
					.flatMap(toNavItem(true))}
				onNavigate={onNavigate}
				output={
					successExample && !step ? (
						<OperationOutputPanel
							summary={successExample.summary}
							status={successExample.status}
							example={successExample.example}
						/>
					) : (
						<NodeOutputPanel
							key={node.id}
							view={
								trigger
									? outputViewFromPayload(sources.lastRun?.triggerPayload)
									: outputViewFromStep(step)
							}
							errorAction={
								<Button asChild variant="outline" size="sm">
									<Link
										to={buildOrgPath(organizationSlug, identificationsPath())}
									>
										Ouvrir les identifications
									</Link>
								</Button>
							}
						/>
					)
				}
			/>
			{picking && (
				<EventPickerFeature
					events={sources.events}
					initialEvent={picking.replacing ?? currentEvent?.name}
					confirmLabel={
						picking.replacing === null && eventNames.length > 0
							? 'Ajouter cet événement'
							: 'Utiliser cet événement'
					}
					onCancel={() => setPicking(null)}
					onConfirm={applyEventChoice}
				/>
			)}
		</>
	)
}

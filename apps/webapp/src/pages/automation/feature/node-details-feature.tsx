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
import {
	collectionPath,
	identificationsPath,
} from '#/pages/automation/lib/automation-paths'
import { branchLabel } from '#/pages/automation/lib/branch-labels'
import { describeSecurity } from '#/pages/automation/lib/collection-format'
import {
	COLLECTION_OPERATION_KIND,
	type CollectionRead,
	flattenOperationConfig,
	lookupOperation,
	nestOperationConfig,
	operationFieldErrors,
	operationForm,
	operationOutput,
} from '#/pages/automation/lib/collection-node'
import { collectionCredentialHint } from '#/pages/automation/lib/connector-field-copy'
import type {
	EditorEdge,
	EditorNode,
	NodeDetailsPatch,
} from '#/pages/automation/lib/editor-nodes'
import { type Neighbour, neighboursOf } from '#/pages/automation/lib/graph'
import { appearanceOf } from '#/pages/automation/lib/node-appearance'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import {
	inputViewFromDryStep,
	type NodeInputView,
	nodeDescription,
	outputViewFromDryStep,
	outputViewFromPayload,
	outputViewFromStep,
} from '#/pages/automation/lib/node-view'
import { labelConflict } from '#/pages/automation/lib/references'
import {
	exampleOf,
	operationContractSource,
} from '#/pages/automation/lib/schema-typescript'
import {
	findEvent,
	triggerEventNames,
} from '#/pages/automation/lib/trigger-events'
import {
	evaluateContextFor,
	type UpstreamSources,
	upstreamEntries,
} from '#/pages/automation/lib/upstream-entries'
import { nodeTextErrors } from '#/pages/automation/lib/validation'
import type { DryRunPasses } from '#/pages/automation/lib/workflow-runs'
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
import { OperationContractTabs } from '#/pages/automation/ui/operation-contract-tabs'
import { OperationOutputPanel } from '#/pages/automation/ui/operation-output-panel'
import { RunStatusLabel } from '#/pages/automation/ui/run-status-label'
import { TriggerParameters } from '#/pages/automation/ui/trigger-parameters'
import type {
	PassSelector,
	TestRun,
} from '#/pages/automation/ui/variables-panel'

export interface NodeDetailsFeatureProps {
	node: EditorNode
	nodes: readonly EditorNode[]
	edges: readonly EditorEdge[]
	organizationSlug: string
	sources: UpstreamSources
	collectionReads: ReadonlyMap<string, CollectionRead>
	authSchemes: Schemas.AuthSchemeResponse[]
	credentials: Schemas.CredentialResponse[]
	issues: readonly NodeIssue[]
	dryRunSteps: DryRunPasses | null
	testRun: TestRun | null
	onUpdateConnector: (
		patch: Partial<
			Pick<Schemas.PlacedConnectorDto, 'config' | 'credential_id'>
		>,
	) => void
	onUpdateTriggerKind: (kind: Schemas.TriggerKindDto) => void
	onUpdateDetails: (patch: NodeDetailsPatch) => void
	onCreateCredential: ConnectorParametersFeatureProps['onCreateCredential']
	onEvaluateExpression: ConnectorParametersFeatureProps['onEvaluateExpression']
	onClose: () => void
	onNavigate: (nodeId: string) => void
}

const NO_PASSES: readonly Schemas.DryRunStepResponse[] = []

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
	collectionReads,
	authSchemes,
	credentials,
	issues,
	dryRunSteps,
	testRun,
	onUpdateConnector,
	onUpdateTriggerKind,
	onUpdateDetails,
	onCreateCredential,
	onEvaluateExpression,
	onClose,
	onNavigate,
}: NodeDetailsFeatureProps) {
	const [picking, setPicking] = useState<{ replacing: string | null } | null>(
		null,
	)
	const [pickedPass, setPickedPass] = useState<{
		nodeId: string
		index: number
	} | null>(null)
	const { connector, trigger } = node.data
	const isOperationNode = connector?.kind === COLLECTION_OPERATION_KIND
	const lookup =
		isOperationNode && connector?.operation
			? lookupOperation(connector.operation, collectionReads)
			: null
	const operation = lookup?.status === 'found' ? lookup.resolved : null
	const form =
		operation && connector ? operationForm(operation, connector.config) : null
	const output = operation ? operationOutput(operation) : null
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
	const step =
		connector && !dryRunSteps ? sources.runSteps.get(connector.id) : undefined
	const passes = (connector && dryRunSteps?.get(connector.id)) || NO_PASSES
	const passIndex =
		pickedPass?.nodeId === node.id
			? Math.min(pickedPass.index, passes.length - 1)
			: passes.length - 1
	const dryStep = passes[passIndex]
	const passSelector: PassSelector | null =
		passes.length > 1
			? {
					index: passIndex,
					count: passes.length,
					onChange: (index) =>
						setPickedPass(
							index >= passes.length - 1 ? null : { nodeId: node.id, index },
						),
				}
			: null
	const executionInput =
		connector && dryRunSteps
			? inputViewFromDryStep(dryStep, node.data.disabled)
			: null
	const shownStatus = dryStep?.status ?? step?.status
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

	const textErrors = nodeTextErrors(
		node.data.errors,
		new Set(descriptor?.fields.map((field) => field.name)),
	)
	const operationErrors = operationFieldErrors(textErrors.rest)
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
							onCommit={(name) => onUpdateDetails({ name })}
							className="font-semibold text-base leading-tight"
						/>
						{textErrors.name && (
							<p role="alert" className="mt-0.5 text-state-alert text-xs">
								{textErrors.name}
							</p>
						)}
						<InlineEditField
							value={node.data.subtitle}
							label="Description du nœud"
							placeholder="Ajouter une description"
							onCommit={(description) => onUpdateDetails({ description })}
							className="text-muted-foreground text-sm"
						/>
						{textErrors.description && (
							<p role="alert" className="mt-0.5 text-state-alert text-xs">
								{textErrors.description}
							</p>
						)}
					</div>
				}
				badge={
					<span className="hidden shrink-0 rounded-sm border border-border px-1.5 py-0.5 text-muted-foreground text-xs sm:inline">
						{node.data.typeLabel}
					</span>
				}
				icon={appearance.icon}
				hue={appearance.hue}
				status={shownStatus && <RunStatusLabel status={shownStatus} />}
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
							execution={executionInput}
							passes={passSelector}
							testRun={testRun}
						/>
					)
				}
				parameters={
					<NodeParametersPanel acceptsExpressions={acceptsExpressions}>
						{isOperationNode && (
							<OperationField
								operation={
									operation
										? {
												method: operation.operation.method,
												path: operation.operation.path,
												summary: operation.operation.summary,
												collectionTitle: operation.collection.title,
												collectionHref: buildOrgPath(
													organizationSlug,
													collectionPath(operation.collection.id),
												),
												baseUrl: operation.collection.baseUrl,
											}
										: null
								}
								unavailable={
									lookup?.status === 'loading'
										? 'loading'
										: lookup?.status === 'unreadable'
											? 'unreadable'
											: 'missing'
								}
								errors={operationErrors.operation}
							/>
						)}
						{connector && descriptor && (!isOperationNode || form) ? (
							<ConnectorParametersFeature
								key={node.id}
								descriptor={
									form ? { ...descriptor, fields: form.fields } : descriptor
								}
								config={
									form
										? flattenOperationConfig(connector.config, form)
										: connector.config
								}
								credentialId={connector.credential_id ?? null}
								credentials={credentials}
								authSchemes={authSchemes}
								errors={form ? operationErrors.fields : textErrors.rest}
								bindingIssues={fieldIssues}
								credentialHint={
									operation
										? collectionCredentialHint(
												operation.collection.security.map(describeSecurity),
											)
										: undefined
								}
								fieldAddOns={form?.copy}
								evaluateContext={evaluateContext}
								onConfigChange={(config) =>
									onUpdateConnector({
										config: form ? nestOperationConfig(config) : config,
									})
								}
								onCredentialChange={(credentialId) =>
									onUpdateConnector({ credential_id: credentialId })
								}
								onCreateCredential={onCreateCredential}
								onEvaluateExpression={onEvaluateExpression}
							/>
						) : trigger ? (
							<>
								{textErrors.rest.length > 0 && (
									<div
										role="alert"
										className="rounded-md border border-state-alert/60 bg-state-alert/10 px-3 py-2 text-sm text-state-alert"
									>
										{textErrors.rest.map((error) => (
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
						{operation && (
							<div className="grid gap-2">
								<p className="font-medium text-sm">Contrat de l’opération</p>
								<OperationContractTabs
									source={operationContractSource(
										operation.collection,
										operation.operation,
									)}
									example={
										operation.operation.response
											? exampleOf(
													operation.operation.response,
													operation.collection.schemas,
												)
											: null
									}
								/>
							</div>
						)}
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
					output && !step && !dryStep ? (
						<OperationOutputPanel
							summary={output.summary}
							status={output.status}
							output={output.output}
						/>
					) : (
						<NodeOutputPanel
							key={node.id}
							passes={passSelector}
							view={
								trigger
									? outputViewFromPayload(sources.lastRun?.triggerPayload)
									: dryStep
										? outputViewFromDryStep(dryStep)
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

import { useEffect, useRef, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Branch,
	type Graph,
	type GraphError,
	type GraphInvalid,
	type PlacedConnector,
	useAutomationCredentials,
	useConnectorCatalogue,
	useCreateCredential,
	useSaveWorkflowVersion,
	useWorkflow,
} from '#/hooks/use-automation'
import { flattenExamplePaths } from '#/pages/automation/lib/expression'
import {
	addConnector,
	addEdge,
	connectorById,
	credentialOptionsFor,
	descriptorFor,
	fieldErrorsFor,
	graphErrorsByConnector,
	groupByFamily,
	matchesAuthRequirement,
	removeConnector,
	removeEdge,
	updateConnectorConfig,
	upstreamOf,
} from '#/pages/automation/lib/graph'
import { layoutPositions } from '#/pages/automation/lib/layout'
import type {
	EditorEdge,
	EditorNode,
	EditorSelection,
	PaletteFamily,
} from '#/pages/automation/ui/workflow-editor-ui'
import { WorkflowEditorUI } from '#/pages/automation/ui/workflow-editor-ui'
import {
	buildCredentialFormErrors,
	emptyCredentialForm,
} from '#/pages/settings/lib/automation'
import type { CredentialFormValues } from '#/pages/settings/types'
import { CredentialFormSheet } from '#/pages/settings/ui/automation-credentials-ui'

const EMPTY_GRAPH: Graph = { connectors: [], edges: [] }

export interface WorkflowEditorFeatureProps {
	workflowId: string
}

type Selection =
	| { type: 'connector'; id: string }
	| { type: 'edge'; from: string; to: string }

export function WorkflowEditorFeature({
	workflowId,
}: WorkflowEditorFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowEditor
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			workflowId={workflowId}
		/>
	)
}

function WorkflowEditor({
	organizationId,
	organizationSlug,
	workflowId,
}: {
	organizationId: string
	organizationSlug: string
	workflowId: string
}) {
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const credentialsQuery = useAutomationCredentials(organizationId)
	const createCredential = useCreateCredential(organizationId)
	const saveVersion = useSaveWorkflowVersion()

	const workflow = workflowQuery.data?.data ?? null
	const catalogue = catalogueQuery.data?.data.connectors ?? []
	const authSchemes = catalogueQuery.data?.data.auth_schemes ?? []
	const credentials = credentialsQuery.data?.data ?? []

	// The graph is edited locally until "Enregistrer" — the backend only
	// gains a new version then (`useSaveWorkflowVersion`'s doc comment: a
	// save is a new version, never an overwrite). Seeded once from the
	// current version, never re-seeded from a background refetch: that
	// would silently discard whatever the user is mid-editing.
	const [graph, setGraph] = useState<Graph | null>(null)
	const seededWorkflowId = useRef<string | null>(null)
	useEffect(() => {
		if (workflow && seededWorkflowId.current !== workflow.id) {
			setGraph(workflow.current_version?.graph ?? EMPTY_GRAPH)
			seededWorkflowId.current = workflow.id
		}
	}, [workflow])

	const [selection, setSelection] = useState<Selection | null>(null)
	const [expressionTargetField, setExpressionTargetField] = useState<
		string | null
	>(null)
	const [graphErrors, setGraphErrors] = useState<GraphError[]>([])
	const [saveError, setSaveError] = useState<string | null>(null)

	const [credentialSheetOpen, setCredentialSheetOpen] = useState(false)
	const [credentialTargetConnectorId, setCredentialTargetConnectorId] =
		useState<string | null>(null)
	const [credentialValues, setCredentialValues] =
		useState<CredentialFormValues>(emptyCredentialForm())
	const [revealedSecret, setRevealedSecret] = useState<string | null>(null)

	const effectiveGraph = graph ?? EMPTY_GRAPH
	const errorsByConnector = graphErrorsByConnector(graphErrors)
	const graphLevelError =
		errorsByConnector
			.get(null)
			?.map((error) => error.message)
			.join(' ') ?? null

	function updateGraph(update: (current: Graph) => Graph) {
		setGraph((current) => update(current ?? EMPTY_GRAPH))
	}

	function handleAddConnector(kind: string) {
		const descriptor = catalogue.find((candidate) => candidate.kind === kind)
		if (!descriptor) return
		const connector: PlacedConnector = {
			id: crypto.randomUUID(),
			kind,
			version: descriptor.version,
			config: {},
			credential_id: null,
		}
		updateGraph((current) => addConnector(current, connector))
		setSelection({ type: 'connector', id: connector.id })
		setExpressionTargetField(null)
	}

	function handleConnect(from: string, to: string) {
		updateGraph((current) => addEdge(current, { from, to }))
	}

	function handleRemoveConnector(connectorId: string) {
		updateGraph((current) => removeConnector(current, connectorId))
		if (selection?.type === 'connector' && selection.id === connectorId) {
			setSelection(null)
		}
	}

	function handleRemoveEdge(from: string, to: string) {
		updateGraph((current) => removeEdge(current, from, to))
		if (
			selection?.type === 'edge' &&
			selection.from === from &&
			selection.to === to
		) {
			setSelection(null)
		}
	}

	function handleBranchChange(from: string, to: string, branch: Branch | null) {
		updateGraph((current) =>
			addEdge(removeEdge(current, from, to), {
				from,
				to,
				branch: branch ?? undefined,
			}),
		)
	}

	function handleFieldChange(
		connectorId: string,
		fieldName: string,
		value: unknown,
	) {
		const connector = connectorById(effectiveGraph, connectorId)
		if (!connector) return
		updateGraph((current) =>
			updateConnectorConfig(current, connectorId, {
				config: { ...connector.config, [fieldName]: value },
			}),
		)
	}

	function handleCredentialChange(
		connectorId: string,
		credentialId: string | null,
	) {
		updateGraph((current) =>
			updateConnectorConfig(current, connectorId, {
				credential_id: credentialId,
			}),
		)
	}

	function handleInsertExpressionValue(expression: string) {
		if (selection?.type !== 'connector' || !expressionTargetField) return
		const connector = connectorById(effectiveGraph, selection.id)
		const current = connector?.config[expressionTargetField]
		const next =
			typeof current === 'string' && current !== ''
				? `${current} ${expression}`
				: expression
		handleFieldChange(selection.id, expressionTargetField, next)
		setExpressionTargetField(null)
	}

	function openCredentialSheet(connectorId: string) {
		const connector = connectorById(effectiveGraph, connectorId)
		const descriptor = connector && descriptorFor(connector, catalogue)
		const defaultKind = authSchemes.find(
			(scheme) =>
				descriptor && matchesAuthRequirement(descriptor.auth, scheme.kind),
		)?.kind
		setCredentialTargetConnectorId(connectorId)
		setCredentialValues(emptyCredentialForm(defaultKind ?? ''))
		setRevealedSecret(null)
		setCredentialSheetOpen(true)
	}

	async function handleCredentialSubmit() {
		const targetConnectorId = credentialTargetConnectorId
		const scheme = authSchemes.find(
			(candidate) => candidate.kind === credentialValues.kind,
		)
		if (
			buildCredentialFormErrors(credentialValues, scheme, 'create').length > 0
		) {
			return
		}

		const created = await createCredential.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				kind: credentialValues.kind,
				name: credentialValues.name.trim(),
				origin: credentialValues.origin,
				data:
					credentialValues.origin === 'supplied'
						? credentialValues.data
						: undefined,
			},
		})

		if (targetConnectorId) {
			handleCredentialChange(targetConnectorId, created.data.id)
		}

		if (credentialValues.origin === 'generated') {
			setRevealedSecret(String(created.data.secret))
		} else {
			setCredentialSheetOpen(false)
		}
	}

	async function handleSave() {
		if (!graph) return
		setGraphErrors([])
		setSaveError(null)
		try {
			await saveVersion.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflowId },
				body: { graph },
			})
		} catch (error) {
			const invalid = asGraphInvalid(error)
			if (invalid) {
				setGraphErrors(invalid.details.errors)
			} else {
				setSaveError(errorMessage(error))
			}
		}
	}

	const nodes: EditorNode[] = effectiveGraph.connectors.map((connector) => {
		const descriptor = descriptorFor(connector, catalogue)
		return {
			id: connector.id,
			label: descriptor?.label ?? connector.kind,
			kindLabel: connector.kind,
			family: descriptor?.family ?? '',
			hasError: errorsByConnector.has(connector.id),
		}
	})

	const edges: EditorEdge[] = effectiveGraph.edges.map((edge) => ({
		from: edge.from,
		to: edge.to,
		branch: edge.branch ?? null,
	}))

	const families: PaletteFamily[] = [...groupByFamily(catalogue)].map(
		([family, connectors]) => ({
			family,
			connectors: connectors.map((connector) => ({
				kind: connector.kind,
				label: connector.label,
			})),
		}),
	)

	const editorSelection: EditorSelection | null = (() => {
		if (!selection) return null
		if (selection.type === 'connector') {
			const connector = connectorById(effectiveGraph, selection.id)
			const descriptor = connector && descriptorFor(connector, catalogue)
			if (!connector) return null
			return {
				type: 'connector',
				data: {
					connectorId: connector.id,
					label: descriptor?.label ?? connector.kind,
					fields: descriptor?.fields ?? [],
					values: connector.config,
					fieldErrors: fieldErrorsFor(graphErrors, connector.id),
					globalError:
						errorsByConnector.get(connector.id)?.find((error) => !error.field)
							?.message ?? null,
					authRequired: descriptor ? descriptor.auth !== 'None' : false,
					credentialOptions: descriptor
						? credentialOptionsFor(descriptor, credentials)
						: [],
					credentialId: connector.credential_id ?? null,
					signingCredentials: credentials
						.filter((credential) => credential.origin === 'generated')
						.map((credential) => ({
							id: credential.id,
							name: credential.name,
						})),
				},
			}
		}

		const edge = effectiveGraph.edges.find(
			(candidate) =>
				candidate.from === selection.from && candidate.to === selection.to,
		)
		if (!edge) return null
		return {
			type: 'edge',
			data: {
				from: edge.from,
				to: edge.to,
				fromLabel: labelForConnector(edge.from),
				toLabel: labelForConnector(edge.to),
				branch: edge.branch ?? null,
			},
		}
	})()

	function labelForConnector(connectorId: string): string {
		const connector = connectorById(effectiveGraph, connectorId)
		if (!connector) return connectorId
		return descriptorFor(connector, catalogue)?.label ?? connector.kind
	}

	const upstreamForExpression =
		editorSelection?.type === 'connector'
			? upstreamOf(effectiveGraph, editorSelection.data.connectorId).map(
					(id) => {
						const connector = connectorById(effectiveGraph, id)
						const descriptor = connector && descriptorFor(connector, catalogue)
						return {
							id,
							label: descriptor?.label ?? connector?.kind ?? id,
							paths: flattenExamplePaths(descriptor?.output_example ?? null),
						}
					},
				)
			: []

	const isLoading =
		workflowQuery.isLoading || catalogueQuery.isLoading || graph === null
	const error =
		workflowQuery.error?.message ?? catalogueQuery.error?.message ?? null

	const compatibleAuthSchemes = (() => {
		if (!credentialTargetConnectorId) return authSchemes
		const connector = connectorById(effectiveGraph, credentialTargetConnectorId)
		const descriptor = connector && descriptorFor(connector, catalogue)
		if (!descriptor) return authSchemes
		return authSchemes.filter((scheme) =>
			matchesAuthRequirement(descriptor.auth, scheme.kind),
		)
	})()

	return (
		<>
			<WorkflowEditorUI
				organizationSlug={organizationSlug}
				workflowId={workflowId}
				workflowName={workflow?.name ?? ''}
				enabled={workflow?.enabled ?? false}
				isLoading={isLoading}
				error={error}
				saveError={saveError ?? graphLevelError}
				isSaving={saveVersion.isPending}
				onSave={() => void handleSave()}
				families={families}
				onAddConnector={handleAddConnector}
				nodes={nodes}
				edges={edges}
				positions={layoutPositions(effectiveGraph)}
				onConnect={handleConnect}
				onSelectConnector={(id) => {
					setSelection({ type: 'connector', id })
					setExpressionTargetField(null)
				}}
				onSelectEdge={(from, to) => {
					setSelection({ type: 'edge', from, to })
					setExpressionTargetField(null)
				}}
				onDeselect={() => {
					setSelection(null)
					setExpressionTargetField(null)
				}}
				selection={editorSelection}
				onFieldChange={handleFieldChange}
				onCredentialChange={handleCredentialChange}
				onRemoveConnector={handleRemoveConnector}
				onCreateCredential={openCredentialSheet}
				onBranchChange={handleBranchChange}
				onRemoveEdge={handleRemoveEdge}
				expressionTargetField={expressionTargetField}
				upstreamForExpression={upstreamForExpression}
				onInsertExpression={setExpressionTargetField}
				onInsertExpressionValue={handleInsertExpressionValue}
				onCloseExpressionPicker={() => setExpressionTargetField(null)}
			/>

			<CredentialFormSheet
				open={credentialSheetOpen}
				mode="create"
				authSchemes={compatibleAuthSchemes}
				form={{
					values: credentialValues,
					isPending: createCredential.isPending,
					onChange: (patch) =>
						setCredentialValues((current) => ({ ...current, ...patch })),
					onSubmit: () => void handleCredentialSubmit(),
				}}
				errors={buildCredentialFormErrors(
					credentialValues,
					authSchemes.find((scheme) => scheme.kind === credentialValues.kind),
					'create',
				)}
				saveError={createCredential.error?.message ?? null}
				revealedSecret={revealedSecret}
				onOpenChange={(open) => {
					setCredentialSheetOpen(open)
					if (!open) setRevealedSecret(null)
				}}
			/>
		</>
	)
}

function errorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return "L'enregistrement a échoué. Réessayez."
}

/** A 422 from the save-version endpoint, structural-checked rather than
 * `instanceof TypedStatusError` — importing `#/api/api.client` (a
 * `*.client.*` file) from code reachable by a route is denied at build
 * time by the SSR bundler's import-protection plugin. Every other API call
 * in this codebase dodges the same wall by going through the
 * `window.tanstackApi` global instead of a static import (see
 * `use-automation.ts`); this is that same workaround applied to reading a
 * thrown error's shape. */
function asGraphInvalid(error: unknown): GraphInvalid | null {
	if (!error || typeof error !== 'object' || !('response' in error)) {
		return null
	}
	const response = (error as { response?: unknown }).response
	if (
		!response ||
		typeof response !== 'object' ||
		(response as { status?: unknown }).status !== 422
	) {
		return null
	}
	return (response as { data: GraphInvalid }).data
}

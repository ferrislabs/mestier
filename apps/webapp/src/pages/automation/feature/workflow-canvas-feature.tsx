import { AlertCircle, Loader2 } from 'lucide-react'
import { useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationCredentials,
	useConnectorCatalogue,
	useCreateCredential,
	useSaveWorkflowVersion,
	useWorkflow,
	useWorkflowTrigger,
	type WorkflowDetail,
} from '#/hooks/use-automation'
import type { NodePosition } from '#/pages/automation/lib/graph'
import { readLayout } from '#/pages/automation/lib/graph'
import { computeFallbackLayout } from '#/pages/automation/lib/layout'
import {
	type GraphValidation,
	projectGraphErrors,
} from '#/pages/automation/lib/validation'
import { WorkflowCanvas } from '#/pages/automation/ui/workflow-canvas'

const EMPTY_VALIDATION: GraphValidation = {
	connectorErrors: new Map(),
	graphErrors: [],
}

export interface WorkflowCanvasFeatureProps {
	workflowId: string
}

export function WorkflowCanvasFeature({
	workflowId,
}: WorkflowCanvasFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<WorkflowCanvasWorkspace
			key={`${activeOrganization.id}:${workflowId}`}
			organizationId={activeOrganization.id}
			workflowId={workflowId}
		/>
	)
}

function extractGraphErrors(
	error: unknown,
): Schemas.GraphErrorResponse[] | null {
	if (!(error instanceof Error)) return null
	const status = (error as { status?: number }).status
	if (status !== 422) return null

	const data = (error as { data?: { details?: { errors?: unknown } } }).data
	const errors = data?.details?.errors
	return Array.isArray(errors) ? (errors as Schemas.GraphErrorResponse[]) : null
}

function descriptorsByKind(
	connectors: Schemas.ConnectorDescriptorResponse[],
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(connectors.map((connector) => [connector.kind, connector]))
}

function mergedLayout(
	graph: Schemas.GraphDto,
	stored: Map<string, NodePosition>,
): Map<string, NodePosition> {
	const fallback = computeFallbackLayout(graph, new Set(stored.keys()))
	return new Map([...fallback, ...stored])
}

function WorkflowCanvasWorkspace({
	organizationId,
	workflowId,
}: {
	organizationId: string
	workflowId: string
}) {
	const workflowQuery = useWorkflow(organizationId, workflowId)
	const catalogueQuery = useConnectorCatalogue(organizationId)
	const triggerQuery = useWorkflowTrigger(organizationId, workflowId)
	const credentialsQuery = useAutomationCredentials(organizationId)

	if (
		workflowQuery.isLoading ||
		catalogueQuery.isLoading ||
		triggerQuery.isLoading ||
		credentialsQuery.isLoading
	) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement du workflow…
				</SectionCard>
			</PageShell>
		)
	}

	if (workflowQuery.isError || !workflowQuery.data?.data) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<div>
						<p className="font-semibold">Impossible de charger le workflow</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{workflowQuery.error?.message ??
								'Réessayez dans quelques instants.'}
						</p>
					</div>
				</SectionCard>
			</PageShell>
		)
	}

	return (
		<WorkflowCanvasLoaded
			organizationId={organizationId}
			workflowId={workflowId}
			workflow={workflowQuery.data.data as WorkflowDetail}
			descriptors={descriptorsByKind(
				catalogueQuery.data?.data.connectors ?? [],
			)}
			authSchemes={catalogueQuery.data?.data.auth_schemes ?? []}
			credentials={credentialsQuery.data?.data ?? []}
			hasTriggerEvent={(triggerQuery.data?.data.event_names.length ?? 0) > 0}
		/>
	)
}

function WorkflowCanvasLoaded({
	organizationId,
	workflowId,
	workflow,
	descriptors,
	authSchemes,
	credentials,
	hasTriggerEvent,
}: {
	organizationId: string
	workflowId: string
	workflow: WorkflowDetail
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	authSchemes: Schemas.AuthSchemeResponse[]
	credentials: Schemas.CredentialResponse[]
	hasTriggerEvent: boolean
}) {
	const saveVersion = useSaveWorkflowVersion()
	const createCredential = useCreateCredential(organizationId)

	const [initial] = useState(() => {
		const graph = workflow.current_version?.graph ?? {
			connectors: [],
			edges: [],
		}
		const layout = mergedLayout(graph, readLayout(workflow.layout))
		return { graph, layout }
	})

	const pending = useRef(initial)
	const [isDirty, setDirty] = useState(false)
	const [validation, setValidation] =
		useState<GraphValidation>(EMPTY_VALIDATION)
	const [saveError, setSaveError] = useState<string | null>(null)

	const handleChange = (
		graph: Schemas.GraphDto,
		layout: Map<string, NodePosition>,
	) => {
		pending.current = { graph, layout }
		setDirty(true)
	}

	const handleCreateCredential = async (
		body: Schemas.CreateCredentialRequest,
	) => {
		const created = await createCredential.mutateAsync({
			path: { organization_id: organizationId },
			body,
		})
		return created.data as Schemas.CredentialResponse & { secret: unknown }
	}

	const handleSave = async () => {
		setValidation(EMPTY_VALIDATION)
		setSaveError(null)
		try {
			await saveVersion.mutateAsync({
				path: { organization_id: organizationId, workflow_id: workflowId },
				body: {
					graph: pending.current.graph,
					layout: Object.fromEntries(pending.current.layout),
				},
			})
			setDirty(false)
		} catch (error) {
			const errors = extractGraphErrors(error)
			if (errors) {
				setValidation(projectGraphErrors(errors))
				return
			}
			setSaveError(
				error instanceof Error && error.message
					? `L’enregistrement a échoué : ${error.message}`
					: 'L’enregistrement a échoué.',
			)
		}
	}

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			{validation.graphErrors.length > 0 || saveError ? (
				<div
					role="alert"
					className="border-b border-destructive/30 bg-destructive/10 px-4 py-2 text-sm text-destructive"
				>
					<ul>
						{saveError ? <li>{saveError}</li> : null}
						{validation.graphErrors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				</div>
			) : null}
			<WorkflowCanvas
				graph={initial.graph}
				layout={initial.layout}
				descriptors={descriptors}
				connectorErrors={validation.connectorErrors}
				hasTriggerEvent={hasTriggerEvent}
				credentials={credentials}
				authSchemes={authSchemes}
				onCreateCredential={handleCreateCredential}
				isDirty={isDirty}
				isSaving={saveVersion.isPending}
				onChange={handleChange}
				onSave={() => void handleSave()}
			/>
		</div>
	)
}

import { useQueryClient } from '@tanstack/react-query'
import {
	useCreateWorkflow,
	useSaveWorkflowVersion,
} from '#/hooks/use-automation'
import { downloadJson } from '#/pages/automation/lib/download'

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'

export interface WorkflowSource {
	id: string
	name: string
	description: string | null
	tags: readonly string[]
}

export function useWorkflowActions(organizationId: string) {
	const queryClient = useQueryClient()
	const createWorkflow = useCreateWorkflow(organizationId)
	const saveVersion = useSaveWorkflowVersion()

	const fetchDetail = (workflowId: string) =>
		queryClient.fetchQuery(
			window.tanstackApi.get(WORKFLOW_PATH, {
				path: { organization_id: organizationId, workflow_id: workflowId },
			}).queryOptions,
		)

	async function duplicate(source: WorkflowSource): Promise<string> {
		const detail = await fetchDetail(source.id)
		const created = await createWorkflow.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				name: `${source.name} (copie)`,
				description: source.description,
			},
		})
		const version = detail.data.current_version
		if (version) {
			await saveVersion.mutateAsync({
				path: {
					organization_id: organizationId,
					workflow_id: created.data.id,
				},
				body: {
					graph: version.graph,
					layout: detail.data.layout ?? undefined,
				},
			})
		}
		return created.data.id
	}

	async function exportWorkflow(source: WorkflowSource): Promise<void> {
		const detail = await fetchDetail(source.id)
		downloadJson(`${source.id}.json`, {
			name: source.name,
			description: source.description,
			tags: source.tags,
			graph: detail.data.current_version?.graph ?? null,
			layout: detail.data.layout ?? null,
		})
	}

	return { duplicate, exportWorkflow }
}

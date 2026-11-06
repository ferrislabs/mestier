import { useMemo } from 'react'
import { useAutomationFolders } from '#/hooks/use-automation'
import {
	toWorkflowFolders,
	type WorkflowFolder,
} from '#/pages/automation/lib/folders'

export function useWorkflowFolders(organizationId: string): {
	data: readonly WorkflowFolder[]
	isLoading: boolean
	error: Error | null
} {
	const query = useAutomationFolders(organizationId)
	const records = query.data?.data
	const data = useMemo(() => toWorkflowFolders(records ?? []), [records])
	return { data, isLoading: query.isLoading, error: query.error }
}

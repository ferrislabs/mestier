import { useMutation, useQueryClient } from '@tanstack/react-query'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { exportFileName } from '#/pages/knowledge-tables/lib/csv-transfer'
import type {
	ImportedKnowledgeTable,
	TableImportReport,
} from '#/pages/knowledge-tables/lib/types'
import { authStore } from '#/store/auth.store'

const TABLES_KEY = 'knowledge-tables'

function tablesPath(organizationId: string): string {
	return `/api/v1/organizations/${organizationId}/knowledge/tables`
}

function messageOf(body: unknown): string | null {
	if (typeof body !== 'object' || body === null || !('message' in body)) {
		return null
	}
	const { message } = body as { message: unknown }
	return typeof message === 'string' ? message : null
}

async function postForm<T>(path: string, form: FormData): Promise<T> {
	const headers = new Headers()
	const accessToken = authStore.state.accessToken
	if (accessToken) headers.set('Authorization', `Bearer ${accessToken}`)
	const response = await fetch(`${window.apiUrl}${path}`, {
		method: 'POST',
		headers,
		credentials: 'include',
		body: form,
	})
	const body: unknown = await response.json().catch(() => null)
	if (!response.ok) {
		throw Object.assign(
			new Error(messageOf(body) ?? `HTTP ${response.status}`),
			{ status: response.status },
		)
	}
	return (body as { data: T }).data
}

function csvForm(file: File, name?: string): FormData {
	const form = new FormData()
	form.append('file', file, file.name)
	if (name !== undefined) form.append('name', name)
	return form
}

export function useImportKnowledgeTable(organizationId: string) {
	const queryClient = useQueryClient()
	return useMutation({
		mutationFn: (input: { file: File; name: string }) =>
			postForm<ImportedKnowledgeTable>(
				`${tablesPath(organizationId)}/import`,
				csvForm(input.file, input.name),
			),
		onSuccess: () => {
			void queryClient.invalidateQueries({
				queryKey: [TABLES_KEY, organizationId],
			})
		},
	})
}

export function useImportTableRows(organizationId: string, tableId: string) {
	const queryClient = useQueryClient()
	return useMutation({
		mutationFn: (file: File) =>
			postForm<TableImportReport>(
				`${tablesPath(organizationId)}/${tableId}/import`,
				csvForm(file),
			),
		onSuccess: () => {
			void queryClient.invalidateQueries({
				queryKey: [TABLES_KEY, organizationId],
			})
		},
	})
}

export function useExportTableView(organizationId: string, tableId: string) {
	return useMutation({
		mutationFn: (input: {
			viewId: string
			tableName: string
			viewName: string
		}) =>
			downloadAuthenticatedFile(
				`${tablesPath(organizationId)}/${tableId}/views/${input.viewId}/export`,
				exportFileName(input.tableName, input.viewName),
			),
	})
}

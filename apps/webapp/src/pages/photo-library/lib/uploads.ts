export type UploadStatus =
	| 'queued'
	| 'converting'
	| 'uploading'
	| 'done'
	| 'duplicate'
	| 'rejected'

export interface UploadItem {
	id: string
	name: string
	status: UploadStatus
	message: string | null
}

export const UPLOAD_CONCURRENCY = 3

export const STATUS_LABELS: Record<UploadStatus, string> = {
	queued: 'En attente',
	converting: 'Conversion…',
	uploading: 'Envoi…',
	done: 'Envoyée',
	duplicate: 'Déjà dans la photothèque',
	rejected: 'Refusée',
}

export function newItem(id: string, name: string): UploadItem {
	return { id, name, status: 'queued', message: null }
}

export function patchItem(
	items: readonly UploadItem[],
	id: string,
	patch: Partial<Omit<UploadItem, 'id'>>,
): UploadItem[] {
	return items.map((item) => (item.id === id ? { ...item, ...patch } : item))
}

export function isActive(item: UploadItem): boolean {
	return (
		item.status === 'queued' ||
		item.status === 'converting' ||
		item.status === 'uploading'
	)
}

export function hasActive(items: readonly UploadItem[]): boolean {
	return items.some(isActive)
}

export function summarize(items: readonly UploadItem[]): string {
	const count = (status: UploadStatus) =>
		items.filter((item) => item.status === status).length
	const parts: string[] = []
	const done = count('done')
	const duplicate = count('duplicate')
	const rejected = count('rejected')
	if (done > 0) parts.push(`${done} envoyée${done > 1 ? 's' : ''}`)
	if (duplicate > 0) parts.push(`${duplicate} en doublon`)
	if (rejected > 0) parts.push(`${rejected} refusée${rejected > 1 ? 's' : ''}`)
	return parts.join(', ')
}

export function clearFinished(items: readonly UploadItem[]): UploadItem[] {
	return items.filter(isActive)
}

export async function runPool<T>(
	tasks: readonly T[],
	limit: number,
	worker: (task: T) => Promise<void>,
): Promise<void> {
	let next = 0
	const lane = async () => {
		while (next < tasks.length) {
			const task = tasks[next]
			next += 1
			await worker(task as T)
		}
	}
	await Promise.all(
		Array.from({ length: Math.min(limit, tasks.length) }, () => lane()),
	)
}

export function uploadErrorMessage(
	status: number,
	serverMessage: string | null,
): string {
	if (status === 413) return 'Fichier trop volumineux.'
	if (status === 422) return 'Image illisible ou format non pris en charge.'
	if (status === 403) {
		return "Vous n'avez pas la permission d'ajouter des photos."
	}
	return serverMessage ?? `Échec de l'envoi (${status}).`
}

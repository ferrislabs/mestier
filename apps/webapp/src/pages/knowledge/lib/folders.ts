import { z } from 'zod'

export interface FolderNode {
	id: string
	parent_id: string | null
	name: string
}

/** A folder at the root is at depth 1. */
export const MAX_FOLDER_DEPTH = 10
export const MAX_FOLDER_NAME_CHARS = 100

/** The type carried by a drag of articles inside the page. */
export const ARTICLE_DRAG_TYPE = 'application/x-mestier-knowledge-articles'

/**
 * `?dossier=<id>`, the folder open in the drive. `.catch` so a hand-edited
 * address lands on the root rather than on a router error page.
 */
export const driveSearchSchema = z.object({
	dossier: z.uuid().optional().catch(undefined),
})

export type DriveSearch = z.infer<typeof driveSearchSchema>

/** The folders of each parent, by name. The root is under the key `null`. */
export function childrenByParent<T extends FolderNode>(
	folders: readonly T[],
): Map<string | null, T[]> {
	const byParent = new Map<string | null, T[]>()
	for (const folder of folders) {
		const siblings = byParent.get(folder.parent_id) ?? []
		siblings.push(folder)
		byParent.set(folder.parent_id, siblings)
	}
	for (const siblings of byParent.values()) {
		siblings.sort((a, b) =>
			a.name.localeCompare(b.name, 'fr', { sensitivity: 'base' }),
		)
	}
	return byParent
}

/** From the root down to the folder, the folder included. Empty when unknown. */
export function pathTo<T extends FolderNode>(
	folders: readonly T[],
	id: string | null,
): T[] {
	const byId = new Map(folders.map((folder) => [folder.id, folder]))
	const path: T[] = []
	const seen = new Set<string>()
	let current = id === null ? undefined : byId.get(id)
	while (current && !seen.has(current.id)) {
		seen.add(current.id)
		path.unshift(current)
		current =
			current.parent_id === null ? undefined : byId.get(current.parent_id)
	}
	return path
}

/** The folder and every folder below it. */
export function subtreeOf(
	folders: readonly FolderNode[],
	id: string,
): Set<string> {
	const byParent = childrenByParent(folders)
	const subtree = new Set<string>([id])
	const queue = [id]
	while (queue.length > 0) {
		const next = queue.shift() as string
		for (const child of byParent.get(next) ?? []) {
			if (!subtree.has(child.id)) {
				subtree.add(child.id)
				queue.push(child.id)
			}
		}
	}
	return subtree
}

/** How many levels hang below the folder: 0 when it has no subfolder. */
export function heightOf(folders: readonly FolderNode[], id: string): number {
	const byParent = childrenByParent(folders)
	const below = (folderId: string): number =>
		Math.max(
			0,
			...(byParent.get(folderId) ?? []).map((child) => 1 + below(child.id)),
		)
	return below(id)
}

/**
 * Where a folder may move: not inside itself, and not so deep that its own
 * tree would pass the depth limit. `null` is the root.
 */
export function canMoveFolderTo(
	folders: readonly FolderNode[],
	moved: string,
	target: string | null,
): boolean {
	if (target === null) return true
	if (subtreeOf(folders, moved).has(target)) return false
	return (
		pathTo(folders, target).length + 1 + heightOf(folders, moved) <=
		MAX_FOLDER_DEPTH
	)
}

/** Whether a new folder fits under `parent` (`null` is the root). */
export function canCreateIn(
	folders: readonly FolderNode[],
	parent: string | null,
): boolean {
	return pathTo(folders, parent).length < MAX_FOLDER_DEPTH
}

/** A refusal to show before sending, or `null` when the name is fine. */
export function folderNameError(
	folders: readonly FolderNode[],
	parent: string | null,
	name: string,
	except: string | null = null,
): string | null {
	const trimmed = name.trim()
	if (trimmed === '') return 'Le nom ne peut pas être vide.'
	if ([...trimmed].length > MAX_FOLDER_NAME_CHARS) {
		return `Le nom fait ${MAX_FOLDER_NAME_CHARS} caractères au plus.`
	}
	const lowered = trimmed.toLocaleLowerCase('fr')
	const taken = folders.some(
		(folder) =>
			folder.parent_id === parent &&
			folder.id !== except &&
			folder.name.toLocaleLowerCase('fr') === lowered,
	)
	return taken ? 'Un dossier porte déjà ce nom ici.' : null
}

export function readDraggedArticles(transfer: DataTransfer): string[] {
	const raw = transfer.getData(ARTICLE_DRAG_TYPE)
	if (!raw) return []
	try {
		const parsed: unknown = JSON.parse(raw)
		return Array.isArray(parsed)
			? parsed.filter((id): id is string => typeof id === 'string')
			: []
	} catch {
		return []
	}
}

export function carriesArticles(transfer: DataTransfer): boolean {
	return Array.from(transfer.types).includes(ARTICLE_DRAG_TYPE)
}

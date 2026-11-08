export interface WorkflowFolder {
	id: string
	name: string
	parentId: string | null
}

interface FolderedWorkflow {
	folderId: string | null
}

interface FolderRecord {
	id: string
	name: string
	parent_id?: string | null
}

export function toWorkflowFolders(
	records: readonly FolderRecord[],
): WorkflowFolder[] {
	return records.map((record) => ({
		id: record.id,
		name: record.name,
		parentId: record.parent_id ?? null,
	}))
}

export function findFolder(
	folders: readonly WorkflowFolder[],
	id: string,
): WorkflowFolder | undefined {
	return folders.find((folder) => folder.id === id)
}

export function childFolders(
	folders: readonly WorkflowFolder[],
	parentId: string | null,
): WorkflowFolder[] {
	return folders
		.filter((folder) => folder.parentId === parentId)
		.sort((left, right) => left.name.localeCompare(right.name, 'fr'))
}

export function folderPath(
	folders: readonly WorkflowFolder[],
	id: string,
): WorkflowFolder[] {
	const path: WorkflowFolder[] = []
	let current = findFolder(folders, id)
	while (current && !path.includes(current)) {
		path.unshift(current)
		current = current.parentId
			? findFolder(folders, current.parentId)
			: undefined
	}
	return path
}

export function isInSubtree(
	folders: readonly WorkflowFolder[],
	folderId: string | null,
	rootId: string,
): boolean {
	if (!folderId) return false
	return folderPath(folders, folderId).some((folder) => folder.id === rootId)
}

export function countWorkflowsInSubtree(
	folders: readonly WorkflowFolder[],
	workflows: readonly FolderedWorkflow[],
	rootId: string,
): number {
	return workflows.filter((workflow) =>
		isInSubtree(folders, workflow.folderId, rootId),
	).length
}

export interface FolderTarget {
	id: string
	label: string
}

export function moveTargets(
	folders: readonly WorkflowFolder[],
	movedId?: string,
): FolderTarget[] {
	return folders
		.filter((folder) => !movedId || !isInSubtree(folders, folder.id, movedId))
		.map((folder) => ({
			id: folder.id,
			label: folderPath(folders, folder.id)
				.map((step) => step.name)
				.join(' › '),
		}))
		.sort((left, right) => left.label.localeCompare(right.label, 'fr'))
}

export interface DragItem {
	kind: 'folder' | 'workflow'
	id: string
	parentId: string | null
}

export function canDropOn(
	folders: readonly WorkflowFolder[],
	item: DragItem,
	targetId: string | null,
): boolean {
	if (targetId === item.parentId) return false
	if (item.kind === 'workflow' || targetId === null) return true
	return !isInSubtree(folders, targetId, item.id)
}

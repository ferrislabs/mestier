export interface WorkflowFolder {
	id: string
	name: string
	parentId: string | null
}

interface FolderedWorkflow {
	folderId: string | null
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

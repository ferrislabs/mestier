import { describe, expect, it } from 'vitest'
import {
	childFolders,
	countWorkflowsInSubtree,
	folderPath,
	isInSubtree,
	type WorkflowFolder,
} from '#/pages/automation/lib/folders'

const FOLDERS: readonly WorkflowFolder[] = [
	{ id: 'sales', name: 'Ventes', parentId: null },
	{ id: 'sales-reminders', name: 'Relances', parentId: 'sales' },
	{ id: 'sales-reminders-quotes', name: 'Devis', parentId: 'sales-reminders' },
	{ id: 'billing', name: 'Facturation', parentId: null },
]

describe('folders', () => {
	it('lists root folders sorted by name', () => {
		expect(childFolders(FOLDERS, null).map((folder) => folder.name)).toEqual([
			'Facturation',
			'Ventes',
		])
	})

	it('builds the path from the root folder down to the folder itself', () => {
		expect(
			folderPath(FOLDERS, 'sales-reminders-quotes').map((f) => f.name),
		).toEqual(['Ventes', 'Relances', 'Devis'])
		expect(folderPath(FOLDERS, 'unknown')).toEqual([])
	})

	it('puts a workflow of a subfolder in every ancestor folder', () => {
		expect(isInSubtree(FOLDERS, 'sales-reminders-quotes', 'sales')).toBe(true)
		expect(isInSubtree(FOLDERS, 'sales-reminders-quotes', 'billing')).toBe(
			false,
		)
		expect(isInSubtree(FOLDERS, null, 'sales')).toBe(false)
	})

	it('counts the subfolders of a root folder in its total', () => {
		const workflows = [
			{ folderId: 'sales' },
			{ folderId: 'sales-reminders-quotes' },
			{ folderId: 'billing' },
			{ folderId: null },
		]
		expect(countWorkflowsInSubtree(FOLDERS, workflows, 'sales')).toBe(2)
		expect(countWorkflowsInSubtree(FOLDERS, workflows, 'billing')).toBe(1)
	})
})

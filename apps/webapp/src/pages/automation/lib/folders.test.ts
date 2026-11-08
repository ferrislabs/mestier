import { describe, expect, it } from 'vitest'
import {
	canDropOn,
	childFolders,
	countWorkflowsInSubtree,
	folderPath,
	isInSubtree,
	moveTargets,
	toWorkflowFolders,
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

describe('toWorkflowFolders', () => {
	it('turns the flat API list into folders, a missing parent meaning the root', () => {
		expect(
			toWorkflowFolders([
				{ id: 'a', name: 'Ventes', parent_id: null },
				{ id: 'b', name: 'Relances', parent_id: 'a' },
				{ id: 'c', name: 'Support' },
			]),
		).toEqual([
			{ id: 'a', name: 'Ventes', parentId: null },
			{ id: 'b', name: 'Relances', parentId: 'a' },
			{ id: 'c', name: 'Support', parentId: null },
		])
	})

	it('builds a tree the other helpers can walk', () => {
		const folders = toWorkflowFolders([
			{ id: 'a', name: 'Ventes', parent_id: null },
			{ id: 'b', name: 'Relances', parent_id: 'a' },
		])

		expect(folderPath(folders, 'b').map((folder) => folder.name)).toEqual([
			'Ventes',
			'Relances',
		])
		expect(countWorkflowsInSubtree(folders, [{ folderId: 'b' }], 'a')).toBe(1)
	})
})

describe('moveTargets', () => {
	it('labels every folder with its path, sorted by label', () => {
		expect(moveTargets(FOLDERS).map((target) => target.label)).toEqual([
			'Facturation',
			'Ventes',
			'Ventes › Relances',
			'Ventes › Relances › Devis',
		])
	})

	it('leaves out the moved folder and everything under it', () => {
		expect(
			moveTargets(FOLDERS, 'sales-reminders').map((target) => target.id),
		).toEqual(['billing', 'sales'])
	})

	it('keeps every folder when nothing is excluded', () => {
		expect(moveTargets(FOLDERS, undefined)).toHaveLength(FOLDERS.length)
	})
})

describe('canDropOn', () => {
	const folder = (id: string, parentId: string | null) => ({
		kind: 'folder' as const,
		id,
		parentId,
	})

	it('refuses the folder the item already sits in', () => {
		expect(
			canDropOn(FOLDERS, folder('sales-reminders', 'sales'), 'sales'),
		).toBe(false)
		expect(canDropOn(FOLDERS, folder('sales', null), null)).toBe(false)
	})

	it('refuses a folder onto itself and onto its own descendants', () => {
		expect(canDropOn(FOLDERS, folder('sales', null), 'sales')).toBe(false)
		expect(
			canDropOn(FOLDERS, folder('sales', null), 'sales-reminders-quotes'),
		).toBe(false)
	})

	it('accepts a folder on a sibling branch or on the root', () => {
		expect(
			canDropOn(FOLDERS, folder('sales-reminders', 'sales'), 'billing'),
		).toBe(true)
		expect(canDropOn(FOLDERS, folder('sales-reminders', 'sales'), null)).toBe(
			true,
		)
	})

	it('lets a workflow go to any other folder, or to the root', () => {
		const item = { kind: 'workflow' as const, id: 'w', parentId: 'sales' }

		expect(canDropOn(FOLDERS, item, 'sales-reminders')).toBe(true)
		expect(canDropOn(FOLDERS, item, null)).toBe(true)
		expect(canDropOn(FOLDERS, item, 'sales')).toBe(false)
	})
})

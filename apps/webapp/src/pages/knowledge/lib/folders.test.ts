import { describe, expect, it } from 'vitest'
import {
	canCreateIn,
	canMoveFolderTo,
	childrenByParent,
	driveSearchSchema,
	type FolderNode,
	folderNameError,
	heightOf,
	pathTo,
	subtreeOf,
} from '#/pages/knowledge/lib/folders'

const folder = (
	id: string,
	parent_id: string | null,
	name = id,
): FolderNode => ({
	id,
	parent_id,
	name,
})

/** a > b > c, and d at the root. */
const tree = [
	folder('c', 'b', 'Parc'),
	folder('a', null, 'Chantiers'),
	folder('b', 'a', '2026'),
	folder('d', null, 'archives'),
]

/** A chain of `depth` folders, n1 at the root. */
function chain(depth: number): FolderNode[] {
	return Array.from({ length: depth }, (_, index) =>
		folder(`n${index + 1}`, index === 0 ? null : `n${index}`),
	)
}

describe('drive folders', () => {
	it('groups the folders by parent, by name whatever the case', () => {
		const byParent = childrenByParent(tree)
		expect(byParent.get(null)?.map((f) => f.name)).toEqual([
			'archives',
			'Chantiers',
		])
		expect(byParent.get('a')?.map((f) => f.id)).toEqual(['b'])
	})

	it('walks the path from the root down to a folder', () => {
		expect(pathTo(tree, 'c').map((f) => f.id)).toEqual(['a', 'b', 'c'])
		expect(pathTo(tree, null)).toEqual([])
		expect(pathTo(tree, 'unknown')).toEqual([])
	})

	it('knows the tree below a folder and its height', () => {
		expect([...subtreeOf(tree, 'a')].sort()).toEqual(['a', 'b', 'c'])
		expect(heightOf(tree, 'a')).toBe(2)
		expect(heightOf(tree, 'c')).toBe(0)
	})

	it('never moves a folder inside itself or past ten levels', () => {
		expect(canMoveFolderTo(tree, 'a', 'c')).toBe(false)
		expect(canMoveFolderTo(tree, 'a', 'a')).toBe(false)
		expect(canMoveFolderTo(tree, 'c', 'd')).toBe(true)
		expect(canMoveFolderTo(tree, 'c', null)).toBe(true)

		const deep = [...chain(9), folder('x', null), folder('y', 'x')]
		expect(canMoveFolderTo(deep, 'x', 'n8')).toBe(true)
		expect(canMoveFolderTo(deep, 'x', 'n9')).toBe(false)
	})

	it('creates up to the tenth level', () => {
		expect(canCreateIn(chain(9), 'n9')).toBe(true)
		expect(canCreateIn(chain(10), 'n10')).toBe(false)
		expect(canCreateIn([], null)).toBe(true)
	})

	it('refuses an empty, too long or taken name among the siblings', () => {
		expect(folderNameError(tree, null, '  ')).not.toBeNull()
		expect(folderNameError(tree, null, 'x'.repeat(101))).not.toBeNull()
		expect(folderNameError(tree, null, ' chantiers ')).toBe(
			'Un dossier porte déjà ce nom ici.',
		)
		expect(folderNameError(tree, 'a', 'Chantiers')).toBeNull()
		expect(folderNameError(tree, null, 'Chantiers', 'a')).toBeNull()
	})

	it('reads the open folder from the address and drops a bad one', () => {
		const id = '019a0000-0000-7000-8000-000000000001'
		expect(driveSearchSchema.parse({ dossier: id })).toEqual({ dossier: id })
		expect(driveSearchSchema.parse({ dossier: 'nope' })).toEqual({
			dossier: undefined,
		})
		expect(driveSearchSchema.parse({})).toEqual({ dossier: undefined })
	})
})

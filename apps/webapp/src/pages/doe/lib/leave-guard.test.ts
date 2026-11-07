import { describe, expect, it } from 'vitest'
import { leavesEditor } from '#/pages/doe/lib/leave-guard'

const doe = '/o/menuiserie-duval/projet/p-lapalisse/doe'

describe('leavesEditor', () => {
	it('stays on the editor when only the tab changes', () => {
		expect(
			leavesEditor({ current: { pathname: doe }, next: { pathname: doe } }),
		).toBe(false)
	})

	it('treats a trailing slash as the same page', () => {
		expect(
			leavesEditor({
				current: { pathname: doe },
				next: { pathname: `${doe}/` },
			}),
		).toBe(false)
	})

	it('leaves the editor when opening the preview', () => {
		expect(
			leavesEditor({
				current: { pathname: doe },
				next: { pathname: `${doe}/preview` },
			}),
		).toBe(true)
	})

	it('leaves the editor for another project or module', () => {
		expect(
			leavesEditor({
				current: { pathname: doe },
				next: {
					pathname: '/o/menuiserie-duval/projet/p-ferry/doe',
				},
			}),
		).toBe(true)
		expect(
			leavesEditor({
				current: { pathname: doe },
				next: { pathname: '/o/menuiserie-duval/crm/clients' },
			}),
		).toBe(true)
	})
})

import { afterEach, describe, expect, it } from 'vitest'
import {
	readNodeMeta,
	writeNodeMeta,
} from '#/pages/automation/feature/mocks/node-meta'

const OPERATION = { collectionId: 'petstore', operationId: 'list' }

afterEach(() => {
	window.localStorage.clear()
})

describe('node meta', () => {
	it('remembers the collection operation of a node per workflow', () => {
		writeNodeMeta('wf-1', { c1: { operation: OPERATION } })

		expect(readNodeMeta('wf-1')).toEqual({ c1: { operation: OPERATION } })
		expect(readNodeMeta('wf-2')).toEqual({})
	})

	it('drops the names, descriptions and flags an earlier version kept here', () => {
		window.localStorage.setItem(
			'mestier:automation:node-meta:wf-1',
			JSON.stringify({
				c1: {
					label: 'Ancien nom',
					subtitle: 'Ancienne description',
					disabled: true,
					operation: OPERATION,
				},
				c2: { label: 'Seulement un nom', disabled: true },
			}),
		)

		expect(readNodeMeta('wf-1')).toEqual({ c1: { operation: OPERATION } })
	})

	it('reads nothing from a corrupted entry', () => {
		window.localStorage.setItem('mestier:automation:node-meta:wf-1', '[1')
		expect(readNodeMeta('wf-1')).toEqual({})

		window.localStorage.setItem('mestier:automation:node-meta:wf-1', '[]')
		expect(readNodeMeta('wf-1')).toEqual({})
	})
})

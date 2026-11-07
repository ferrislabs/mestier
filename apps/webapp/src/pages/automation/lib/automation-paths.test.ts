import { describe, expect, it } from 'vitest'
import {
	automationBasePath,
	collectionPath,
	collectionsPath,
	executionsPath,
	identificationsPath,
	legacyExecutionLocation,
	NEW_WORKFLOW_ID,
	newWorkflowPath,
	parseExecutionNumber,
	parseWorkflowSearch,
	projectPath,
	workflowExecutionsLocation,
	workflowExecutionsSearch,
	workflowPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'

describe('automation paths', () => {
	it('builds the module entry points relative to the organization', () => {
		expect(automationBasePath()).toBe('/automatisation')
		expect(workflowsPath()).toBe('/automatisation/workflows')
		expect(executionsPath()).toBe('/automatisation/executions')
		expect(collectionsPath()).toBe('/automatisation/collections')
		expect(identificationsPath()).toBe('/automatisation/identifications')
	})

	it('builds the paths of a single workflow, a draft included', () => {
		expect(workflowPath('wf-1')).toBe('/automatisation/workflows/wf-1')
		expect(newWorkflowPath()).toBe('/automatisation/workflows/nouveau')
		expect(NEW_WORKFLOW_ID).toBe('nouveau')
		expect(newWorkflowPath()).toBe(workflowPath(NEW_WORKFLOW_ID))
	})

	it('builds the path of a collection and of a project', () => {
		expect(collectionPath('col-1')).toBe('/automatisation/collections/col-1')
		expect(projectPath('ventes')).toBe('/automatisation/projets/ventes')
	})

	it('puts the executions view of a workflow in the search, not in the path', () => {
		expect(workflowExecutionsSearch()).toEqual({ vue: 'executions' })
		expect(workflowExecutionsLocation('wf-1')).toEqual({
			to: '/automatisation/workflows/wf-1',
			search: { vue: 'executions' },
		})
	})

	it('keeps the number of the selected run in the search when there is one', () => {
		expect(workflowExecutionsSearch(1285)).toEqual({
			vue: 'executions',
			execution: 1285,
		})
		expect(workflowExecutionsLocation('wf-1', 1285)).toEqual({
			to: '/automatisation/workflows/wf-1',
			search: { vue: 'executions', execution: 1285 },
		})
	})

	it('leaves out a missing or impossible run number', () => {
		expect(workflowExecutionsSearch(undefined)).toEqual({ vue: 'executions' })
		expect(workflowExecutionsSearch(0)).toEqual({ vue: 'executions' })
		expect(workflowExecutionsSearch(-4)).toEqual({ vue: 'executions' })
		expect(workflowExecutionsSearch(1.5)).toEqual({ vue: 'executions' })
		expect(workflowExecutionsSearch(Number.NaN)).toEqual({ vue: 'executions' })
	})
})

describe('parseExecutionNumber', () => {
	it('keeps a positive whole number, given as a number or as digits', () => {
		expect(parseExecutionNumber(1285)).toBe(1285)
		expect(parseExecutionNumber('1285')).toBe(1285)
		expect(parseExecutionNumber('7')).toBe(7)
	})

	it('rejects everything else', () => {
		for (const value of [
			'',
			' ',
			'run-1',
			'1285a',
			'12.5',
			'-3',
			'0',
			'1e3',
			' 12',
			'018f6c1e-7b1a-4d2e-9c11-5f3a2b4c6d7e',
			0,
			-3,
			1.5,
			Number.NaN,
			Number.POSITIVE_INFINITY,
			null,
			undefined,
			true,
			{},
		]) {
			expect(parseExecutionNumber(value)).toBeUndefined()
		}
	})
})

describe('legacyExecutionLocation', () => {
	it('sends an old run link to the executions view, the run selected when its id is a number', () => {
		expect(legacyExecutionLocation('wf-1', '1285')).toEqual({
			to: '/automatisation/workflows/wf-1',
			search: { vue: 'executions', execution: 1285 },
		})
	})

	it('sends an old link that carries a UUID to the executions view alone', () => {
		expect(
			legacyExecutionLocation('wf-1', '018f6c1e-7b1a-4d2e-9c11-5f3a2b4c6d7e'),
		).toEqual({
			to: '/automatisation/workflows/wf-1',
			search: { vue: 'executions' },
		})
	})
})

describe('parseWorkflowSearch', () => {
	it('keeps the executions view and the number of a run', () => {
		expect(parseWorkflowSearch({ vue: 'executions', execution: 1285 })).toEqual(
			{
				vue: 'executions',
				execution: 1285,
			},
		)
	})

	it('converts the digits the address bar hands over', () => {
		expect(
			parseWorkflowSearch({ vue: 'executions', execution: '1285' }),
		).toEqual({ vue: 'executions', execution: 1285 })
	})

	it('keeps a run number without a view', () => {
		expect(parseWorkflowSearch({ execution: 12 })).toEqual({ execution: 12 })
	})

	it('drops any other view', () => {
		expect(parseWorkflowSearch({ vue: 'editeur' })).toEqual({})
		expect(parseWorkflowSearch({ vue: true })).toEqual({})
	})

	it('drops a run identifier that is not a positive whole number', () => {
		for (const execution of ['', 'run-1', 0, -3, 1.5, '12.5', null, true]) {
			expect(parseWorkflowSearch({ execution })).toEqual({})
		}
	})

	it('ignores unknown keys', () => {
		expect(parseWorkflowSearch({ nouveau: '1' })).toEqual({})
	})
})

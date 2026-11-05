import { describe, expect, it } from 'vitest'
import {
	automationBasePath,
	collectionPath,
	collectionsPath,
	executionsPath,
	identificationsPath,
	NEW_WORKFLOW_ID,
	newWorkflowPath,
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

	it('keeps the selected run in the search when there is one', () => {
		expect(workflowExecutionsSearch('run-1')).toEqual({
			vue: 'executions',
			execution: 'run-1',
		})
		expect(workflowExecutionsLocation('wf-1', 'run-1')).toEqual({
			to: '/automatisation/workflows/wf-1',
			search: { vue: 'executions', execution: 'run-1' },
		})
	})

	it('leaves out an empty run identifier', () => {
		expect(workflowExecutionsSearch('')).toEqual({ vue: 'executions' })
		expect(workflowExecutionsSearch(undefined)).toEqual({ vue: 'executions' })
	})
})

describe('parseWorkflowSearch', () => {
	it('keeps the executions view and a non-empty run identifier', () => {
		expect(
			parseWorkflowSearch({ vue: 'executions', execution: 'run-1' }),
		).toEqual({ vue: 'executions', execution: 'run-1' })
	})

	it('keeps a run identifier without a view', () => {
		expect(parseWorkflowSearch({ execution: 'run-1' })).toEqual({
			execution: 'run-1',
		})
	})

	it('drops any other view', () => {
		expect(parseWorkflowSearch({ vue: 'editeur' })).toEqual({})
		expect(parseWorkflowSearch({ vue: true })).toEqual({})
	})

	it('drops an empty or non-string run identifier', () => {
		expect(parseWorkflowSearch({ execution: '' })).toEqual({})
		expect(parseWorkflowSearch({ execution: 12 })).toEqual({})
		expect(parseWorkflowSearch({ execution: null })).toEqual({})
	})

	it('ignores unknown keys', () => {
		expect(parseWorkflowSearch({ nouveau: '1' })).toEqual({})
	})
})

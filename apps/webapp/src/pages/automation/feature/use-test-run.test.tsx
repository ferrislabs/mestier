import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { STEP_MS, useTestRun } from '#/pages/automation/feature/use-test-run'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'

const DRY_RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/dry-run'

function connector(id: string): Schemas.PlacedConnectorDto {
	return { id, kind: 'http.request', version: 1, config: {} }
}

function graphOf(...ids: string[]): Schemas.GraphDto {
	return { connectors: ids.map(connector), edges: [], triggers: [] }
}

function step(
	connectorId: string,
	patch: Partial<Schemas.DryRunStepResponse> = {},
): Schemas.DryRunStepResponse {
	return {
		connector_id: connectorId,
		iteration_path: '',
		status: 'succeeded',
		input: null,
		output: null,
		error: null,
		...patch,
	}
}

function refusal(status: number, message: string, errors?: unknown) {
	return Object.assign(new Error(message), {
		status,
		data: errors === undefined ? undefined : { details: { errors } },
	})
}

function setup(
	respond: (params: unknown) => unknown,
	initial: Schemas.GraphDto = graphOf('c1'),
) {
	const api = installFakeTanstackApi()
	api.mockMutation('post', DRY_RUN_PATH, respond)
	const client = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	let current = initial
	const wrapper = ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>{children}</QueryClientProvider>
	)
	const rendered = renderHook(
		() =>
			useTestRun({
				organizationId: 'org-1',
				currentGraph: () => current,
			}),
		{ wrapper },
	)
	return {
		api,
		...rendered,
		setGraph: (graph: Schemas.GraphDto) => {
			current = graph
		},
	}
}

async function settle() {
	await act(async () => {
		await vi.advanceTimersByTimeAsync(0)
	})
}

async function elapse(ms: number) {
	await act(async () => {
		await vi.advanceTimersByTimeAsync(ms)
	})
}

async function startAndAnswer(result: { current: { start: () => void } }) {
	act(() => result.current.start())
	await settle()
}

function keysOf(result: {
	current: { steps: ReadonlyMap<string, unknown> | null }
}) {
	return [...(result.current.steps?.keys() ?? [])]
}

function answering(...steps: Schemas.DryRunStepResponse[]) {
	return () => ({ data: { steps }, pagination: null })
}

beforeEach(() => {
	vi.useFakeTimers()
})

afterEach(() => {
	vi.useRealTimers()
})

describe('useTestRun — the request', () => {
	it('sends the current graph of the editor, not a saved one, to the dry-run endpoint', async () => {
		const { api, result, setGraph } = setup(answering(step('c1')))
		const edited = graphOf('c1', 'c2')
		setGraph(edited)

		await startAndAnswer(result)

		expect(api.calls).toEqual([
			{
				method: 'post',
				path: DRY_RUN_PATH,
				params: {
					path: { organization_id: 'org-1' },
					body: { graph: edited },
				},
			},
		])
	})

	it('runs while it waits for the answer, with nothing to show yet', () => {
		const { result } = setup(() => new Promise(() => {}))

		act(() => result.current.start())

		expect(result.current.running).toBe(true)
		expect(result.current.steps).toBeNull()
	})

	it('refuses a second start while one is going', async () => {
		const { api, result } = setup(answering(step('c1')))

		act(() => {
			result.current.start()
			result.current.start()
		})
		await settle()
		act(() => result.current.start())
		await settle()

		expect(api.calls).toHaveLength(1)
	})
})

describe('useTestRun — the animation', () => {
	it('clears every status on the answer, then reveals one step every 350 ms in the order of the answer', async () => {
		const { result } = setup(
			answering(
				step('c2', { output: { n: 2 } }),
				step('c1', { output: { n: 1 } }),
				step('c3', { status: 'skipped' }),
			),
		)

		await startAndAnswer(result)

		expect(STEP_MS).toBe(350)
		expect(result.current.running).toBe(true)
		expect(keysOf(result)).toEqual([])

		await elapse(349)
		expect(keysOf(result)).toEqual([])

		await elapse(1)
		expect(keysOf(result)).toEqual(['c2'])

		await elapse(350)
		expect(keysOf(result)).toEqual(['c2', 'c1', 'c3'])
		expect(result.current.steps?.get('c3')?.[0]?.status).toBe('skipped')
		expect(result.current.running).toBe(true)

		await elapse(350)
		expect(result.current.running).toBe(false)
		expect(result.current.steps?.size).toBe(3)
	})

	it('spends no beat on a node that did not run: it shows along with the step before it', async () => {
		const { result } = setup(
			answering(
				step('c1'),
				step('c2', { status: 'skipped' }),
				step('c3'),
				step('c4', { status: 'skipped' }),
			),
		)

		await startAndAnswer(result)
		await elapse(STEP_MS)
		expect(keysOf(result)).toEqual(['c1', 'c2'])

		await elapse(STEP_MS)
		expect(keysOf(result)).toEqual(['c1', 'c2', 'c3', 'c4'])
		expect(result.current.running).toBe(true)

		await elapse(STEP_MS)
		expect(result.current.running).toBe(false)
	})

	it('stops at the first failure: the failed node and everything left unreached show together', async () => {
		const { result } = setup(
			answering(
				step('c1', { output: 1 }),
				step('c2', { status: 'failed', error: 'URL introuvable' }),
				step('c3', { status: 'skipped' }),
				step('c4', { status: 'skipped' }),
			),
		)

		await startAndAnswer(result)
		await elapse(STEP_MS)
		expect(keysOf(result)).toEqual(['c1'])

		await elapse(STEP_MS)
		expect(keysOf(result)).toEqual(['c1', 'c2', 'c3', 'c4'])
		expect(result.current.steps?.get('c2')?.[0]).toMatchObject({
			status: 'failed',
			error: 'URL introuvable',
		})
		expect(result.current.steps?.get('c3')?.[0]).toMatchObject({
			status: 'skipped',
			input: null,
			output: null,
			error: null,
		})

		await elapse(STEP_MS)
		expect(result.current.running).toBe(false)
	})

	it('shows what is at the head of the answer and did not run at once', async () => {
		const { result } = setup(
			answering(step('c1', { status: 'skipped' }), step('c2')),
		)

		await startAndAnswer(result)

		expect(keysOf(result)).toEqual(['c1'])
	})

	it('keeps every pass of a loop body apart, indexed by node and iteration path, and reveals them one by one', async () => {
		const { result } = setup(
			answering(
				step('c1'),
				step('c2', { iteration_path: 'c1[0]', output: 'one' }),
				step('c2', { iteration_path: 'c1[1]', output: 'two' }),
				step('c2', { iteration_path: 'c1[2]', output: 'three' }),
			),
		)

		await startAndAnswer(result)
		await elapse(STEP_MS * 2)
		expect(result.current.steps?.get('c2')?.map((pass) => pass.output)).toEqual(
			['one'],
		)

		await elapse(STEP_MS)
		expect(result.current.steps?.get('c2')?.map((pass) => pass.output)).toEqual(
			['one', 'two'],
		)

		await elapse(STEP_MS)
		expect(
			result.current.steps?.get('c2')?.map((pass) => pass.iteration_path),
		).toEqual(['c1[0]', 'c1[1]', 'c1[2]'])
		expect(result.current.steps?.get('c1')).toHaveLength(1)
	})

	it('ends at once on a graph with nothing reachable, after the beat', async () => {
		const { result } = setup(answering())

		await startAndAnswer(result)

		expect(result.current.steps?.size).toBe(0)
		expect(result.current.running).toBe(true)

		await elapse(STEP_MS)
		expect(result.current.running).toBe(false)
		expect(result.current.steps?.size).toBe(0)
	})

	it('starts afresh on a new test: the previous results go at once', async () => {
		const { result } = setup(answering(step('c1', { output: 1 })))
		await startAndAnswer(result)
		await elapse(STEP_MS * 2)
		expect(result.current.steps?.size).toBe(1)

		act(() => result.current.start())

		expect(result.current.steps).toBeNull()
		expect(result.current.running).toBe(true)
		await settle()
		expect(result.current.steps?.size).toBe(0)
	})

	it('stops its timers when the screen goes away', async () => {
		const { result, unmount } = setup(answering(step('c1'), step('c2')))
		await startAndAnswer(result)
		const clear = vi.spyOn(window, 'clearTimeout')

		unmount()

		expect(clear).toHaveBeenCalledTimes(3)
		clear.mockRestore()
	})
})

describe('useTestRun — the graph changes', () => {
	it('keeps the results when the graph is the same, whatever the key order', async () => {
		const { result } = setup(answering(step('c1')))
		await startAndAnswer(result)
		await elapse(STEP_MS * 2)

		act(() =>
			result.current.graphChanged({
				triggers: [],
				edges: [],
				connectors: [
					{ config: {}, version: 1, kind: 'http.request', id: 'c1' },
				],
			}),
		)

		expect(result.current.steps?.size).toBe(1)
	})

	it('drops the results as soon as the graph differs', async () => {
		const { result } = setup(answering(step('c1')))
		await startAndAnswer(result)
		await elapse(STEP_MS * 2)

		act(() => result.current.graphChanged(graphOf('c1', 'c2')))

		expect(result.current.steps).toBeNull()
		expect(result.current.running).toBe(false)
	})

	it('stops an animation under way', async () => {
		const { result } = setup(answering(step('c1'), step('c2'), step('c3')))
		await startAndAnswer(result)
		await elapse(STEP_MS)

		act(() => result.current.graphChanged(graphOf('c1', 'c2')))
		await elapse(STEP_MS * 5)

		expect(result.current.steps).toBeNull()
		expect(result.current.running).toBe(false)
	})

	it('ignores the answer to a graph that has changed since', async () => {
		let answer: (value: unknown) => void = () => {}
		const { result } = setup(
			() =>
				new Promise((resolve) => {
					answer = resolve
				}),
		)
		act(() => result.current.start())

		act(() => result.current.graphChanged(graphOf('c1', 'c2')))
		await act(async () => {
			answer({ data: { steps: [step('c1')] }, pagination: null })
			await vi.advanceTimersByTimeAsync(STEP_MS * 3)
		})

		expect(result.current.steps).toBeNull()
		expect(result.current.running).toBe(false)
	})

	it('allows a new test once the graph changed during the previous one', async () => {
		const { api, result } = setup(answering(step('c1')))
		await startAndAnswer(result)
		act(() => result.current.graphChanged(graphOf('c1', 'c2')))

		await startAndAnswer(result)

		expect(api.calls).toHaveLength(2)
	})
})

describe('useTestRun — the refusals', () => {
	const INVALID = [
		{ connector_id: 'c1', field: 'url', message: 'URL manquante' },
		{ connector_id: 'c2', field: null, message: 'Identification manquante' },
		{ connector_id: null, trigger_id: null, field: null, message: 'cycle' },
	]

	function refuseWithGraphErrors() {
		return setup(
			() => {
				throw refusal(422, 'Invalid graph', INVALID)
			},
			graphOf('c1', 'c2'),
		)
	}

	it('turns a 422 E_GRAPH_INVALID into the validation of a save, and no error message', async () => {
		const { result } = refuseWithGraphErrors()

		await startAndAnswer(result)

		expect(result.current.running).toBe(false)
		expect(result.current.steps).toBeNull()
		expect(result.current.error).toBeNull()
		expect(result.current.validation.connectorErrors.get('c1')).toEqual([
			{ field: 'url', message: 'URL manquante' },
		])
		expect(result.current.validation.connectorErrors.get('c2')).toEqual([
			{ field: null, message: 'Identification manquante' },
		])
		expect(result.current.validation.graphErrors).toEqual(['cycle'])
	})

	it('lifts the badge of the node that changed and leaves the others', async () => {
		const { result } = refuseWithGraphErrors()
		await startAndAnswer(result)

		act(() =>
			result.current.graphChanged({
				connectors: [
					{ id: 'c1', kind: 'http.request', version: 1, config: { url: 'a' } },
					connector('c2'),
				],
				edges: [],
				triggers: [],
			}),
		)

		expect([...result.current.validation.connectorErrors.keys()]).toEqual([
			'c2',
		])
		expect(result.current.validation.graphErrors).toEqual(['cycle'])
	})

	it('wipes the previous validation when a test starts', async () => {
		const { result } = refuseWithGraphErrors()
		await startAndAnswer(result)

		act(() => result.current.start())

		expect(result.current.validation.connectorErrors.size).toBe(0)
		expect(result.current.validation.graphErrors).toEqual([])
		await settle()
	})

	it.each([
		[400, 'Failed to parse the request body as JSON'],
		[403, 'Forbidden'],
		[413, 'Payload Too Large'],
		[422, 'trigger `ghost` is not in this graph'],
		[503, 'Service Unavailable'],
	])('says a %i without graph errors as a message', async (status, message) => {
		const { result } = setup(() => {
			throw refusal(status, message)
		})

		await startAndAnswer(result)

		expect(result.current.error).toBe(
			`Le test du workflow a échoué : ${message}`,
		)
		expect(result.current.running).toBe(false)
		expect(result.current.steps).toBeNull()
		expect(result.current.validation.connectorErrors.size).toBe(0)
	})

	it('says the static message of an E_UNPROCESSABLE_ENTITY as it is', async () => {
		const { result } = setup(() => {
			throw Object.assign(new Error('a dry run walks at most 1000 steps'), {
				status: 422,
				data: {
					code: 'E_UNPROCESSABLE_ENTITY',
					message: 'a dry run walks at most 1000 steps',
					status: 422,
				},
			})
		})

		await startAndAnswer(result)

		expect(result.current.error).toBe(
			'Le test du workflow a échoué : a dry run walks at most 1000 steps',
		)
		expect(result.current.validation.connectorErrors.size).toBe(0)
		expect(result.current.validation.graphErrors).toEqual([])
	})

	it('still says something when an E_GRAPH_INVALID names no error', async () => {
		const { result } = setup(() => {
			throw refusal(422, 'Invalid graph', [])
		})

		await startAndAnswer(result)

		expect(result.current.error).toBe(
			'Le test du workflow a échoué : Invalid graph',
		)
	})

	it('clears the message with the next test, then with a change of the graph', async () => {
		let attempts = 0
		const { result } = setup(() => {
			attempts += 1
			if (attempts === 1) throw refusal(503, 'Service Unavailable')
			if (attempts === 2) throw refusal(503, 'Service Unavailable')
			return { data: { steps: [] }, pagination: null }
		})
		await startAndAnswer(result)
		expect(result.current.error).not.toBeNull()

		act(() => result.current.start())
		expect(result.current.error).toBeNull()
		await settle()
		expect(result.current.error).not.toBeNull()

		act(() => result.current.graphChanged(graphOf('c1', 'c2')))
		expect(result.current.error).toBeNull()
	})
})

// MOCK: execution node counts — the runs list carries no step summary. Replace with `steps_done` and `steps_total` on the run response.
export interface ExecutionNodeCounts {
	done: number
	total: number
}

function hash(value: string): number {
	let result = 0
	for (const char of value) result = (result * 31 + char.charCodeAt(0)) >>> 0
	return result
}

const MIN_NODES = 3
const SPREAD = 5

export function mockNodeCounts(run: {
	id: string
	workflow_id: string
	status: string
}): ExecutionNodeCounts {
	const total = MIN_NODES + (hash(run.workflow_id) % SPREAD)
	switch (run.status) {
		case 'succeeded':
			return { done: total, total }
		case 'failed':
			return { done: Math.max(1, hash(run.id) % total), total }
		case 'running':
			return { done: Math.max(1, hash(run.id) % total), total }
		default:
			return { done: 0, total }
	}
}

import type { RunStep } from '#/hooks/use-automation'

/**
 * One loop iteration's steps, sharing the same non-empty `iteration_path`
 * (see `libs/core/src/domain/automation/run/graph.rs`'s
 * `format_iteration_path`: `""` outside any loop, `"loop1[0]"` inside one,
 * `"loop1[0].loop2[1]"` nested — outermost loop first).
 */
export interface IterationGroup {
	/** The exact backend path shared by every step in this group. */
	path: string
	/** French label built from the path's loop indices, one segment per
	 * nesting level, e.g. "Itération 0" or "Itération 0 › Itération 1". */
	label: string
	steps: RunStep[]
}

export type RunStepBlock =
	| { kind: 'step'; step: RunStep }
	| { kind: 'iteration'; group: IterationGroup }

/**
 * Chunks a run's steps — already in graph/execution order, never resorted
 * here — into presentation blocks: a step outside any loop is its own
 * block, and steps sharing a loop iteration are grouped under one
 * collapsible block instead of listed flat.
 *
 * Grouping only merges *consecutive* steps with the same `iteration_path`.
 * The engine walks a loop's body to completion before moving on, so one
 * iteration's steps are always contiguous in the returned order; if the same
 * path ever resurfaced later out of sequence, treating it as a second,
 * distinct block is the only option that does not silently reorder steps.
 */
export function groupRunSteps(steps: RunStep[]): RunStepBlock[] {
	const blocks: RunStepBlock[] = []
	let openGroup: IterationGroup | null = null

	for (const step of steps) {
		if (step.iteration_path === '') {
			openGroup = null
			blocks.push({ kind: 'step', step })
			continue
		}

		if (openGroup && openGroup.path === step.iteration_path) {
			openGroup.steps.push(step)
			continue
		}

		openGroup = {
			path: step.iteration_path,
			label: formatIterationLabel(step.iteration_path),
			steps: [step],
		}
		blocks.push({ kind: 'iteration', group: openGroup })
	}

	return blocks
}

function formatIterationLabel(path: string): string {
	return path
		.split('.')
		.map((segment) => {
			const match = segment.match(/\[(\d+)\]$/)
			return `Itération ${match ? match[1] : segment}`
		})
		.join(' › ')
}

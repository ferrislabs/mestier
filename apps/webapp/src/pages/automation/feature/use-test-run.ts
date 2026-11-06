import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useDryRunWorkflow } from '#/hooks/use-automation'
import { sameGraph } from '#/pages/automation/lib/graph'
import {
	type GraphValidation,
	graphErrorsOf,
	projectGraphErrors,
	pruneCorrectedErrors,
} from '#/pages/automation/lib/validation'
import {
	type DryRunPasses,
	dryRunPassesByConnector,
	dryRunReveals,
} from '#/pages/automation/lib/workflow-runs'

export const STEP_MS = 350

const TEST_FAILED = 'Le test du workflow a échoué'

const NO_VALIDATION: GraphValidation = {
	connectorErrors: new Map(),
	graphErrors: [],
}

interface Outcome {
	steps: readonly Schemas.DryRunStepResponse[]
	revealed: number
}

export interface TestRunState {
	running: boolean
	steps: DryRunPasses | null
	error: string | null
	validation: GraphValidation
	start: () => void
	graphChanged: (graph: Schemas.GraphDto) => void
}

function failure(error: unknown): string {
	return error instanceof Error && error.message
		? `${TEST_FAILED} : ${error.message}`
		: `${TEST_FAILED}.`
}

export function useTestRun({
	organizationId,
	currentGraph,
}: {
	organizationId: string
	currentGraph: () => Schemas.GraphDto
}): TestRunState {
	const { mutateAsync: dryRun } = useDryRunWorkflow()
	const [running, setRunning] = useState(false)
	const [outcome, setOutcome] = useState<Outcome | null>(null)
	const [error, setError] = useState<string | null>(null)
	const [validation, setValidation] = useState(NO_VALIDATION)
	const timers = useRef<number[]>([])
	const attempt = useRef(0)
	const busy = useRef(false)
	const tested = useRef<Schemas.GraphDto | null>(null)

	const stop = useCallback(() => {
		attempt.current += 1
		busy.current = false
		for (const timer of timers.current) window.clearTimeout(timer)
		timers.current = []
	}, [])

	useEffect(() => stop, [stop])

	const start = useCallback(() => {
		if (busy.current) return
		const graph = currentGraph()
		stop()
		busy.current = true
		const token = attempt.current
		tested.current = graph
		setRunning(true)
		setError(null)
		setValidation(NO_VALIDATION)
		setOutcome(null)
		dryRun({ path: { organization_id: organizationId }, body: { graph } }).then(
			(response) => {
				if (attempt.current !== token) return
				const steps = response.data.steps
				const { lead, beats } = dryRunReveals(steps)
				setOutcome({ steps, revealed: lead })
				timers.current = [
					...beats.map((revealed, beat) =>
						window.setTimeout(
							() => setOutcome({ steps, revealed }),
							STEP_MS * (beat + 1),
						),
					),
					window.setTimeout(
						() => {
							busy.current = false
							setRunning(false)
						},
						STEP_MS * (beats.length + 1),
					),
				]
			},
			(refusal: unknown) => {
				if (attempt.current !== token) return
				busy.current = false
				const errors = graphErrorsOf(refusal)
				if (errors) setValidation(projectGraphErrors(errors))
				else setError(failure(refusal))
				setRunning(false)
			},
		)
	}, [dryRun, organizationId, currentGraph, stop])

	const graphChanged = useCallback(
		(graph: Schemas.GraphDto) => {
			const baseline = tested.current
			if (!baseline || sameGraph(baseline, graph)) return
			stop()
			setRunning(false)
			setOutcome(null)
			setError(null)
			setValidation((current) => pruneCorrectedErrors(current, baseline, graph))
		},
		[stop],
	)

	const steps = useMemo(
		() =>
			outcome
				? dryRunPassesByConnector(outcome.steps.slice(0, outcome.revealed))
				: null,
		[outcome],
	)

	return { running, steps, error, validation, start, graphChanged }
}

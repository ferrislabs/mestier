import { useEffect, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useEvaluateExpression } from '#/hooks/use-automation'
import {
	displayValue,
	type ParameterEntry,
} from '#/pages/automation/lib/run-node'
import type { ExecutionParameterEntry } from '#/pages/automation/ui/execution-parameters-panel'

const UNRESOLVED = 'Non résolue : ce nœud n’a reçu aucun élément.'

export function useResolvedParameters({
	organizationId,
	entries,
	context,
	hasItem,
}: {
	organizationId: string
	entries: readonly ParameterEntry[]
	context: Schemas.EvaluateContextBody
	hasItem: boolean
}): ExecutionParameterEntry[] {
	const evaluate = useEvaluateExpression()
	const [resolved, setResolved] = useState<ReadonlyMap<string, string>>(
		new Map(),
	)
	const key = JSON.stringify({
		items: entries
			.filter((entry) => entry.expression)
			.map((entry) => ({ id: entry.id, template: entry.template })),
		context,
	})
	const { mutateAsync } = evaluate

	useEffect(() => {
		const { items, context: evaluationContext } = JSON.parse(key) as {
			items: { id: string; template: unknown }[]
			context: Schemas.EvaluateContextBody
		}
		let cancelled = false
		if (items.length === 0 || !hasItem) {
			setResolved(new Map())
			return
		}
		void Promise.all(
			items.map(async (item): Promise<[string, string]> => {
				try {
					const result = await mutateAsync({
						path: { organization_id: organizationId },
						body: { template: item.template, context: evaluationContext },
					})
					return [
						item.id,
						displayValue((result.data as { value: unknown }).value),
					]
				} catch (caught) {
					return [
						item.id,
						`Non résolue : ${caught instanceof Error && caught.message ? caught.message : 'évaluation impossible.'}`,
					]
				}
			}),
		).then((pairs) => {
			if (!cancelled) setResolved(new Map(pairs))
		})
		return () => {
			cancelled = true
		}
	}, [key, hasItem, organizationId, mutateAsync])

	return entries.map((entry) => ({
		id: entry.id,
		label: entry.label,
		value: entry.value,
		resolved: !entry.expression
			? null
			: !hasItem
				? UNRESOLVED
				: (resolved.get(entry.id) ?? 'Évaluation en cours…'),
	}))
}

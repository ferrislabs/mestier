import type { LucideIcon } from 'lucide-react'
import {
	CONDITION_ENTRY,
	END_ENTRY,
	type NodeChoiceEntry,
	nodeChoices,
	questionKindEntry,
} from '#/pages/survey/lib/question-kinds'
import type { GraphNode } from '#/pages/survey/types'

export interface NodeChoiceGroup {
	id: 'questions' | 'flow'
	label: string
	entries: NodeChoiceEntry[]
}

function normalize(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
		.trim()
}

export function nodeChoiceGroups(query: string): NodeChoiceGroup[] {
	const needle = normalize(query)
	const matches = (entry: NodeChoiceEntry) =>
		needle === '' ||
		normalize(`${entry.title} ${entry.description}`).includes(needle)
	const entries = nodeChoices().filter(matches)
	const isFlow = (entry: NodeChoiceEntry) =>
		entry.choice === 'CONDITION' || entry.choice === 'END'
	const groups: NodeChoiceGroup[] = [
		{
			id: 'questions',
			label: 'Questions',
			entries: entries.filter((entry) => !isFlow(entry)),
		},
		{ id: 'flow', label: 'Parcours', entries: entries.filter(isFlow) },
	]
	return groups.filter((group) => group.entries.length > 0)
}

export interface NodeAppearance {
	icon: LucideIcon
	hue: number
	typeLabel: string
	description: string
}

export function nodeAppearance(node: GraphNode): NodeAppearance {
	const entry =
		node.type === 'QUESTION'
			? questionKindEntry(node.kind)
			: node.type === 'CONDITION'
				? CONDITION_ENTRY
				: END_ENTRY
	return {
		icon: entry.icon,
		hue: entry.hue,
		typeLabel: entry.title,
		description: entry.description,
	}
}

export function nodeTitle(node: GraphNode | undefined): string {
	if (!node) return 'Nœud'
	if (node.type === 'QUESTION') {
		return node.label.trim() === ''
			? `${questionKindEntry(node.kind).title} sans libellé`
			: node.label
	}
	if (node.type === 'CONDITION') return 'Condition'
	return 'Fin'
}

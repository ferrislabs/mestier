import { isChoiceKind } from '#/pages/survey/lib/question-kinds'
import {
	blockEnd,
	type DisplayRule,
	type FormQuestion,
	type RuleJoin,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import { normalizeGraph } from '#/pages/survey/lib/survey-graph'
import type {
	Branch,
	ConditionNode,
	EditorGraph,
	GraphEdge,
	GraphNode,
	QuestionNode,
} from '#/pages/survey/types'

export type AdvancedReason =
	| 'no_start'
	| 'no_end'
	| 'several_ends'
	| 'unknown_node'
	| 'dead_end'
	| 'cycle'
	| 'shape'

export type ParsedGraph =
	| { kind: 'form'; form: SurveyForm }
	| { kind: 'advanced'; reason: AdvancedReason; questionCount: number }

function cyrb128(text: string): number[] {
	let h1 = 1779033703
	let h2 = 3144134277
	let h3 = 1013904242
	let h4 = 2773480762
	for (let index = 0; index < text.length; index += 1) {
		const code = text.charCodeAt(index)
		h1 = h2 ^ Math.imul(h1 ^ code, 597399067)
		h2 = h3 ^ Math.imul(h2 ^ code, 2869860233)
		h3 = h4 ^ Math.imul(h3 ^ code, 951274213)
		h4 = h1 ^ Math.imul(h4 ^ code, 2716044179)
	}
	h1 = Math.imul(h3 ^ (h1 >>> 18), 597399067)
	h2 = Math.imul(h4 ^ (h2 >>> 22), 2869860233)
	h3 = Math.imul(h1 ^ (h3 >>> 17), 951274213)
	h4 = Math.imul(h2 ^ (h4 >>> 19), 2716044179)
	h1 ^= h2 ^ h3 ^ h4
	h2 ^= h1
	h3 ^= h1
	h4 ^= h1
	return [h1 >>> 0, h2 >>> 0, h3 >>> 0, h4 >>> 0]
}

export function conditionNodeId(questionId: string, position: number): string {
	const hex = cyrb128(`${questionId}:condition:${position}`)
		.map((part) => part.toString(16).padStart(8, '0'))
		.join('')
	const variant = ((Number.parseInt(hex[16], 16) & 0x3) | 0x8).toString(16)
	return [
		hex.slice(0, 8),
		hex.slice(8, 12),
		`8${hex.slice(13, 16)}`,
		`${variant}${hex.slice(17, 20)}`,
		hex.slice(20, 32),
	].join('-')
}

export function conditionOwners(form: SurveyForm): Map<string, string> {
	const owners = new Map<string, string>()
	for (const item of form.questions) {
		item.rule?.conditions.forEach((_, position) => {
			owners.set(conditionNodeId(item.id, position), item.id)
		})
	}
	return owners
}

function questionNode(item: FormQuestion): QuestionNode {
	return {
		type: 'QUESTION',
		id: item.id,
		kind: item.kind,
		label: item.label,
		required: item.required,
		options: isChoiceKind(item.kind) ? item.options : [],
		max_photos: item.kind === 'PHOTO' ? item.maxPhotos : null,
		identifier: item.identifier,
		...(item.block ? { block: item.block } : {}),
	}
}

export function blocksWellFormed(questions: readonly FormQuestion[]): boolean {
	const closed = new Set<string>()
	let position = 0
	while (position < questions.length) {
		const block = questions[position].block
		const last = blockEnd(questions, position)
		if (block !== null) {
			if (closed.has(block) || last === position) return false
			if (questions.slice(position + 1, last + 1).some((item) => item.rule)) {
				return false
			}
			closed.add(block)
		}
		position = last + 1
	}
	return true
}

function entryOf(form: SurveyForm, position: number): string {
	const item = form.questions[position]
	if (!item) return form.endId
	return item.rule && item.rule.conditions.length > 0
		? conditionNodeId(item.id, 0)
		: item.id
}

export function buildGraph(form: SurveyForm): EditorGraph {
	const end: GraphNode = {
		type: 'END',
		id: form.endId,
		message: form.endMessage === '' ? null : form.endMessage,
	}
	if (form.questions.length === 0) {
		return { start: null, nodes: [end], edges: [] }
	}
	const nodes: GraphNode[] = []
	const edges: GraphEdge[] = []
	form.questions.forEach((item, position) => {
		const after = entryOf(form, position + 1)
		const skip = entryOf(form, blockEnd(form.questions, position) + 1)
		const conditions = item.rule?.conditions ?? []
		conditions.forEach((condition, at) => {
			const id = conditionNodeId(item.id, at)
			nodes.push({
				type: 'CONDITION',
				id,
				question: condition.question,
				option: condition.option,
				checked: condition.checked,
			})
			const following =
				at + 1 < conditions.length ? conditionNodeId(item.id, at + 1) : null
			if (item.rule?.join === 'ANY') {
				edges.push({ from: id, to: item.id, branch: 'THEN' })
				edges.push({ from: id, to: following ?? skip, branch: 'ELSE' })
			} else {
				edges.push({ from: id, to: following ?? item.id, branch: 'THEN' })
				edges.push({ from: id, to: skip, branch: 'ELSE' })
			}
		})
		nodes.push(questionNode(item))
		edges.push({ from: item.id, to: after, branch: 'NEXT' })
	})
	nodes.push(end)
	return { start: entryOf(form, 0), nodes, edges }
}

function formQuestion(
	node: QuestionNode,
	rule: DisplayRule | null,
): FormQuestion {
	return {
		id: node.id,
		kind: node.kind,
		label: node.label,
		required: node.required,
		options: isChoiceKind(node.kind) ? [...(node.options ?? [])] : [],
		maxPhotos: node.kind === 'PHOTO' ? (node.max_photos ?? null) : null,
		identifier: Boolean(node.identifier),
		rule,
		block: node.block ?? null,
	}
}

function signature(graph: EditorGraph): string {
	const normalized = normalizeGraph(graph)
	return JSON.stringify({
		start: normalized.start,
		nodes: [...normalized.nodes].sort((a, b) => a.id.localeCompare(b.id)),
		edges: [...normalized.edges]
			.map((edge) => ({ from: edge.from, to: edge.to, branch: edge.branch }))
			.sort((a, b) =>
				`${a.from}:${a.branch}:${a.to}`.localeCompare(
					`${b.from}:${b.branch}:${b.to}`,
				),
			),
	})
}

function renamed(graph: EditorGraph, names: Map<string, string>): EditorGraph {
	const rename = (id: string) => names.get(id) ?? id
	return {
		start: graph.start ? rename(graph.start) : null,
		nodes: graph.nodes.map((node) => ({ ...node, id: rename(node.id) })),
		edges: graph.edges.map((edge) => ({
			...edge,
			from: rename(edge.from),
			to: rename(edge.to),
		})),
	}
}

interface Chain {
	join: RuleJoin
	conditions: ConditionNode[]
	members: QuestionNode[]
	after: string
}

export function parseGraph(graph: EditorGraph): ParsedGraph {
	const questionCount = graph.nodes.filter(
		(node) => node.type === 'QUESTION',
	).length
	const advanced = (reason: AdvancedReason): ParsedGraph => ({
		kind: 'advanced',
		reason,
		questionCount,
	})
	const ends = graph.nodes.filter((node) => node.type === 'END')
	if (ends.length > 1) return advanced('several_ends')
	if (!graph.start) {
		if (graph.nodes.some((node) => node.type !== 'END')) {
			return advanced('no_start')
		}
		const end = ends[0]
		if (!end) return advanced('no_end')
		return {
			kind: 'form',
			form: { questions: [], endId: end.id, endMessage: end.message ?? '' },
		}
	}
	const end = ends[0]
	if (!end) return advanced('no_end')
	const byId = new Map(graph.nodes.map((node) => [node.id, node]))
	const next = (id: string, branch: Branch) =>
		graph.edges.find((edge) => edge.from === id && edge.branch === branch)?.to
	const limit = graph.nodes.length + 1

	const follow = (start: string, branch: Branch, target?: string) => {
		const conditions: ConditionNode[] = []
		let cursor: string | undefined = start
		while (cursor && conditions.length < limit) {
			const node = byId.get(cursor)
			if (node?.type !== 'CONDITION') break
			if (target !== undefined && next(node.id, 'THEN') !== target) break
			conditions.push(node)
			cursor = next(node.id, branch)
		}
		return { conditions, stop: cursor }
	}

	const membersFrom = (first: QuestionNode) => {
		const members = [first]
		let cursor = next(first.id, 'NEXT')
		while (first.block && cursor && members.length < limit) {
			const node = byId.get(cursor)
			if (node?.type !== 'QUESTION' || node.block !== first.block) break
			members.push(node)
			cursor = next(node.id, 'NEXT')
		}
		return { members, after: cursor }
	}

	const chainAt = (start: string): Chain | null => {
		const all = follow(start, 'THEN')
		const allTarget = all.stop ? byId.get(all.stop) : undefined
		if (allTarget?.type === 'QUESTION') {
			const { members, after } = membersFrom(allTarget)
			if (
				after &&
				all.conditions.every((node) => next(node.id, 'ELSE') === after)
			) {
				return { join: 'ALL', conditions: all.conditions, members, after }
			}
		}
		const thenId = next(start, 'THEN')
		const anyTarget = thenId ? byId.get(thenId) : undefined
		if (anyTarget?.type !== 'QUESTION') return null
		const any = follow(start, 'ELSE', anyTarget.id)
		const { members, after } = membersFrom(anyTarget)
		if (!any.stop || after !== any.stop) return null
		return { join: 'ANY', conditions: any.conditions, members, after }
	}

	const questions: FormQuestion[] = []
	const names = new Map<string, string>()
	const seen = new Set<string>()
	let cursor: string = graph.start
	while (true) {
		if (seen.has(cursor)) return advanced('cycle')
		const node = byId.get(cursor)
		if (!node) return advanced('unknown_node')
		if (node.type === 'END') break
		if (node.type === 'QUESTION') {
			seen.add(node.id)
			questions.push(formQuestion(node, null))
			const following = next(node.id, 'NEXT')
			if (!following) return advanced('dead_end')
			cursor = following
			continue
		}
		const chain = chainAt(node.id)
		if (!chain || chain.members.some((member) => seen.has(member.id))) {
			return advanced('shape')
		}
		const [target, ...rest] = chain.members
		for (const [position, condition] of chain.conditions.entries()) {
			if (seen.has(condition.id)) return advanced('cycle')
			seen.add(condition.id)
			names.set(condition.id, conditionNodeId(target.id, position))
		}
		for (const member of chain.members) seen.add(member.id)
		questions.push(
			formQuestion(target, {
				join: chain.join,
				conditions: chain.conditions.map((condition) => ({
					question: condition.question,
					option: condition.option,
					checked: condition.checked,
				})),
			}),
			...rest.map((member) => formQuestion(member, null)),
		)
		cursor = chain.after
	}
	if (!blocksWellFormed(questions)) return advanced('shape')
	const form: SurveyForm = {
		questions,
		endId: end.id,
		endMessage: end.message ?? '',
	}
	if (signature(renamed(graph, names)) !== signature(buildGraph(form))) {
		return advanced('shape')
	}
	return { kind: 'form', form }
}

export const LEGACY_SYNTAX_MESSAGE =
	"Cette expression utilise l’ancienne syntaxe connectors.<id>.output : utilise $('Nom du nœud').item.json…"

export function unknownNodeMessage(label: string): string {
	return `Le nœud « ${label} » n’existe plus.`
}

export function notUpstreamMessage(label: string): string {
	return `Le nœud « ${label} » n’est pas relié en amont de ce nœud.`
}

export function missingKeyMessage(path: string, label: string): string {
	return `La clé « ${path} » est absente de la sortie de « ${label} ».`
}

const LEGACY_SERVER_MESSAGE = /ancienne syntaxe `connectors\.<id>\.output`/
const UNKNOWN_NODE =
	/^connector `[^`]*` field `[^`]*` references unknown node `(.*)`$/
const NODE_NOT_UPSTREAM =
	/^connector `[^`]*` field `[^`]*` references node `(.*)`, which does not run before it$/
const NO_OUTPUT = /^node `(.*)` has no output available to this expression$/
const NO_ITEM = /^node `(.*)` produced no item$/

export function translateReferenceError(message: string): string {
	if (LEGACY_SERVER_MESSAGE.test(message)) return LEGACY_SYNTAX_MESSAGE
	const unknown = UNKNOWN_NODE.exec(message)
	if (unknown) return unknownNodeMessage(unknown[1])
	const notUpstream = NODE_NOT_UPSTREAM.exec(message)
	if (notUpstream) return notUpstreamMessage(notUpstream[1])
	return message
}

export function previewErrorMessage(message: string): string {
	if (LEGACY_SERVER_MESSAGE.test(message)) return LEGACY_SYNTAX_MESSAGE
	const noOutput = NO_OUTPUT.exec(message)
	if (noOutput) {
		return `Le nœud « ${noOutput[1]} » n’existe pas ou n’est pas relié en amont de ce nœud.`
	}
	const noItem = NO_ITEM.exec(message)
	if (noItem) return `Le nœud « ${noItem[1]} » n’a produit aucun élément.`
	return message
}

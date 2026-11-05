import type { ReactNode } from 'react'

const TOKEN_PATTERN =
	/\b(export|interface|type)\b|'[^']*'|\b([A-Z]\w*)\b|^(\s+)(\w+)(?=:)|\b(string|number|null)\b/gm

function highlight(source: string): ReactNode[] {
	const nodes: ReactNode[] = []
	let cursor = 0
	for (const match of source.matchAll(TOKEN_PATTERN)) {
		const start = match.index
		if (start > cursor) nodes.push(source.slice(cursor, start))
		const [token, keyword, typeName, indent, field, primitive] = match
		if (keyword) {
			nodes.push(
				<span key={start} className="text-muted-foreground">
					{token}
				</span>,
			)
		} else if (field) {
			nodes.push(
				indent,
				<span key={start} className="text-foreground">
					{field}
				</span>,
			)
		} else if (typeName || primitive) {
			nodes.push(
				<span key={start} className="font-medium text-highlight">
					{token}
				</span>,
			)
		} else {
			nodes.push(
				<span key={start} className="text-state-today">
					{token}
				</span>,
			)
		}
		cursor = start + token.length
	}
	if (cursor < source.length) nodes.push(source.slice(cursor))
	return nodes
}

export function TypeScriptView({ source }: { source: string }) {
	return (
		<pre className="overflow-x-auto rounded-md border border-border bg-card p-3 font-mono text-xs leading-5">
			<code>{highlight(source)}</code>
		</pre>
	)
}

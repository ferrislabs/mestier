import type { ReactNode } from 'react'

const TOKEN_PATTERN =
	/("(?:\\.|[^"\\])*")(\s*:)?|\b(?:true|false|null)\b|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g

function highlight(source: string): ReactNode[] {
	const nodes: ReactNode[] = []
	let cursor = 0
	for (const match of source.matchAll(TOKEN_PATTERN)) {
		const start = match.index
		if (start > cursor) nodes.push(source.slice(cursor, start))
		const [token, quoted, colon] = match
		if (quoted && colon) {
			nodes.push(
				<span key={start} className="text-muted-foreground">
					{quoted}
				</span>,
				colon,
			)
		} else if (quoted) {
			nodes.push(
				<span key={start} className="text-foreground">
					{token}
				</span>,
			)
		} else {
			nodes.push(
				<span key={start} className="font-medium text-highlight">
					{token}
				</span>,
			)
		}
		cursor = start + token.length
	}
	if (cursor < source.length) nodes.push(source.slice(cursor))
	return nodes
}

export function JsonView({ value }: { value: unknown }) {
	return (
		<pre className="overflow-x-auto rounded-md border border-border bg-card p-3 font-mono text-xs leading-5 whitespace-pre">
			{highlight(JSON.stringify(value, null, 2) ?? 'null')}
		</pre>
	)
}

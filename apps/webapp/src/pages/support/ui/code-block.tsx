import { Check, Copy } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '#/components/ui/button'

export function CodeBlock({ code, label }: { code: string; label: string }) {
	const [copied, setCopied] = useState(false)
	useEffect(() => {
		if (!copied) return
		const timer = window.setTimeout(() => setCopied(false), 1500)
		return () => window.clearTimeout(timer)
	}, [copied])
	return (
		<figure aria-label={label} className="relative">
			<pre className="overflow-x-auto rounded-md border border-border bg-muted/50 p-3 pr-11 font-mono text-xs leading-5">
				{code}
			</pre>
			<Button
				type="button"
				variant="ghost"
				size="icon-sm"
				className="absolute top-1.5 right-1.5"
				aria-label={`Copier : ${label}`}
				onClick={() => {
					navigator.clipboard?.writeText(code).catch(() => undefined)
					setCopied(true)
				}}
			>
				{copied ? <Check /> : <Copy />}
			</Button>
		</figure>
	)
}

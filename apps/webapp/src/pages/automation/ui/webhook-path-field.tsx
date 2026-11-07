import { Check, Copy } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	normalizeWebhookPath,
	WEBHOOK_PREFIX,
	webhookUrl,
} from '#/pages/automation/lib/webhook-path'

const COPIED_MS = 1500

export function WebhookPathField({
	controlId,
	value,
	invalid,
	origin,
	onChange,
}: {
	controlId: string
	value: string
	invalid: boolean
	origin: string
	onChange: (value: string) => void
}) {
	const [copied, setCopied] = useState(false)
	const url = webhookUrl(origin, value)

	useEffect(() => {
		if (!copied) return
		const timer = window.setTimeout(() => setCopied(false), COPIED_MS)
		return () => window.clearTimeout(timer)
	}, [copied])

	return (
		<div className="grid gap-1.5">
			<div
				className={cn(
					'flex min-w-0 items-stretch overflow-hidden rounded-md border border-input shadow-xs transition-[color,box-shadow] focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50 dark:bg-input/30',
					invalid && 'border-state-alert',
				)}
			>
				<span
					className="flex shrink-0 items-center border-input border-r bg-muted px-2.5 font-mono text-muted-foreground text-sm"
					title="Préfixe imposé"
				>
					/{WEBHOOK_PREFIX}/
				</span>
				<input
					id={controlId}
					value={value}
					onChange={(event) => onChange(event.target.value)}
					onBlur={() => {
						const normalized = normalizeWebhookPath(value)
						if (normalized !== value) onChange(normalized)
					}}
					placeholder="stripe/paiement-recu"
					spellCheck={false}
					autoComplete="off"
					aria-invalid={invalid || undefined}
					className="min-w-0 flex-1 bg-transparent px-2.5 py-1.5 font-mono text-sm outline-none placeholder:text-muted-foreground"
				/>
			</div>
			<div className="flex min-w-0 items-center gap-1.5">
				<code className="min-w-0 flex-1 truncate font-mono text-muted-foreground text-xs">
					{url}
				</code>
				<Button
					type="button"
					variant="ghost"
					size="icon-xs"
					aria-label="Copier l’URL du webhook"
					onClick={() => {
						navigator.clipboard?.writeText(url).catch(() => undefined)
						setCopied(true)
					}}
				>
					{copied ? <Check /> : <Copy />}
				</Button>
			</div>
		</div>
	)
}

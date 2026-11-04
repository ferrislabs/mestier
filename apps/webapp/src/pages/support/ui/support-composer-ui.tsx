import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Textarea } from '#/components/ui/textarea'
import type { SupportComposerUIProps } from '#/pages/support/types'

export function SupportComposerUI({
	tabs,
	activeTab,
	body,
	isSubmitting,
	error,
	onTabChange,
	onBodyChange,
	onSubmit,
}: SupportComposerUIProps) {
	const activeTabModel = tabs.find((tab) => tab.id === activeTab)
	const isBlank = body.trim() === ''

	function handleSubmit() {
		if (isBlank || isSubmitting) return
		onSubmit()
	}

	return (
		<div className="flex flex-col gap-3 border-t p-5">
			<div
				role="tablist"
				aria-label="Type de message"
				className="flex flex-wrap gap-1"
			>
				{tabs.map((tab) => (
					<button
						key={tab.id}
						type="button"
						role="tab"
						aria-selected={tab.id === activeTab}
						onClick={() => onTabChange(tab.id)}
						className={`rounded-md px-3 py-1.5 text-sm font-medium outline-none transition focus-visible:ring-2 focus-visible:ring-ring ${
							tab.id === activeTab
								? 'bg-brand-soft text-primary'
								: 'text-muted-foreground hover:bg-muted'
						}`}
					>
						{tab.label}
					</button>
				))}
			</div>

			<Textarea
				aria-label="Message"
				rows={4}
				value={body}
				onChange={(event) => onBodyChange(event.target.value)}
			/>

			{activeTabModel ? (
				<p className="text-xs text-muted-foreground">{activeTabModel.hint}</p>
			) : null}

			{error ? <p className="text-sm text-destructive">{error}</p> : null}

			<div className="flex justify-end">
				<Button
					type="button"
					disabled={isBlank || isSubmitting}
					onClick={handleSubmit}
				>
					{isSubmitting ? (
						<Loader2 aria-hidden="true" className="animate-spin" />
					) : null}
					{activeTabModel?.actionLabel ?? 'Envoyer'}
				</Button>
			</div>
		</div>
	)
}

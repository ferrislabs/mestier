import { AlertCircle, Check, Copy, Loader2, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	STATUS_LABELS,
	summarize,
	type UploadItem,
	type UploadStatus,
} from '#/pages/photo-library/lib/uploads'

interface UploadListProps {
	items: readonly UploadItem[]
	isBusy: boolean
	onClear: () => void
}

function StatusIcon({ status }: { status: UploadStatus }) {
	switch (status) {
		case 'done':
			return <Check aria-hidden className="size-4 text-state-ok" />
		case 'duplicate':
			return <Copy aria-hidden className="size-4 text-muted-foreground" />
		case 'rejected':
			return <AlertCircle aria-hidden className="size-4 text-destructive" />
		default:
			return (
				<Loader2
					aria-hidden
					className="size-4 animate-spin text-muted-foreground"
				/>
			)
	}
}

export function UploadList({ items, isBusy, onClear }: UploadListProps) {
	if (items.length === 0) return null
	return (
		<section aria-label="Envois en cours" className="grid gap-2">
			<div className="flex items-center justify-between gap-2">
				<p className="text-muted-foreground text-sm">
					{isBusy ? 'Envoi en cours…' : summarize(items)}
				</p>
				<Button
					type="button"
					variant="ghost"
					size="sm"
					disabled={isBusy}
					onClick={onClear}
				>
					<X aria-hidden />
					Effacer la liste
				</Button>
			</div>
			<ul className="divide-y rounded-md border">
				{items.map((item) => (
					<li key={item.id} className="flex items-center gap-3 px-3 py-2">
						<StatusIcon status={item.status} />
						<span className="min-w-0 flex-1 truncate text-sm">{item.name}</span>
						<span
							className={
								item.status === 'rejected'
									? 'text-destructive text-xs'
									: 'text-muted-foreground text-xs'
							}
						>
							{item.message ?? STATUS_LABELS[item.status]}
						</span>
					</li>
				))}
			</ul>
		</section>
	)
}

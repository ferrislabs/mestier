import {
	Banknote,
	BellRing,
	CircleCheck,
	CircleX,
	FileMinus,
	FilePlus,
	FileText,
	GitBranch,
	type LucideIcon,
	Pencil,
	Send,
	StickyNote,
} from 'lucide-react'
import { formatDocumentDateTime } from '#/pages/quotes/lib/doc-format'

export type HistoryKind =
	| 'created'
	| 'updated'
	| 'issued'
	| 'sent'
	| 'accepted'
	| 'declined'
	| 'revised'
	| 'reminder'
	| 'payment'
	| 'credit_note'
	| 'note'

export interface HistoryEntry {
	id: string
	kind: HistoryKind
	detail: string
	at: string
}

const ICONS: Record<HistoryKind, LucideIcon> = {
	created: FilePlus,
	updated: Pencil,
	issued: FileText,
	sent: Send,
	accepted: CircleCheck,
	declined: CircleX,
	revised: GitBranch,
	reminder: BellRing,
	payment: Banknote,
	credit_note: FileMinus,
	note: StickyNote,
}

export function HistoryList({ entries }: { entries: readonly HistoryEntry[] }) {
	if (entries.length === 0)
		return <p className="text-muted-foreground text-sm">Aucun événement.</p>
	const ordered = [...entries].sort((a, b) => b.at.localeCompare(a.at))
	return (
		<ol className="grid">
			{ordered.map((entry, index) => {
				const Icon = ICONS[entry.kind]
				return (
					<li key={entry.id} className="relative flex gap-3 pb-4 last:pb-0">
						{index < ordered.length - 1 && (
							<span
								aria-hidden
								className="absolute top-7 bottom-0 left-3.5 w-px bg-border"
							/>
						)}
						<span className="relative flex size-7 shrink-0 items-center justify-center rounded-full border bg-card text-muted-foreground">
							<Icon className="size-3.5" aria-hidden />
						</span>
						<div className="min-w-0 pt-0.5">
							<p className="text-sm">{entry.detail}</p>
							<p className="text-muted-foreground text-xs">
								<time dateTime={entry.at}>
									{formatDocumentDateTime(entry.at)}
								</time>
							</p>
						</div>
					</li>
				)
			})}
		</ol>
	)
}

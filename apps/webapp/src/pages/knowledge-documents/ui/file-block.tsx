import { Download, FileText, Loader2, Paperclip } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	BLOCK_TOOL_BUTTON,
	BLOCK_TOOLBAR,
} from '#/pages/knowledge-documents/ui/block-toolbar'
import { DeletedElement } from '#/pages/knowledge-documents/ui/deleted-element'

export type FileBlockState = 'empty' | 'loading' | 'ready' | 'missing'

interface FileBlockProps {
	state: FileBlockState
	title: string
	detail: string
	editable: boolean
	isDownloading: boolean
	onPick: () => void
	onDownload: () => void
}

export function FileBlock({
	state,
	title,
	detail,
	editable,
	isDownloading,
	onPick,
	onDownload,
}: FileBlockProps) {
	if (state === 'empty' && editable) {
		return (
			<button
				type="button"
				onClick={onPick}
				className="flex w-full items-center gap-3 rounded-lg bg-muted/50 px-4 py-3 text-left text-muted-foreground text-sm outline-none transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50"
			>
				<Paperclip aria-hidden className="size-4" />
				Joindre un fichier du Drive
			</button>
		)
	}
	return (
		<div className="group/block relative flex w-full items-center gap-3 rounded-lg border px-3 py-2.5 text-sm transition-colors hover:bg-muted/40">
			<span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
				<FileText aria-hidden className="size-4" />
			</span>
			<div className="min-w-0 flex-1">
				{state === 'empty' ? (
					<span className="text-muted-foreground">Aucun fichier choisi.</span>
				) : state === 'missing' ? (
					<DeletedElement />
				) : state === 'loading' ? (
					<Loader2 aria-hidden className="size-4 animate-spin" />
				) : (
					<>
						<span className="block truncate font-medium">{title}</span>
						<span className="block text-muted-foreground text-xs">
							{detail}
						</span>
					</>
				)}
			</div>
			{editable ? (
				<div
					contentEditable={false}
					className={cn(BLOCK_TOOLBAR, 'shadow-none')}
				>
					<button type="button" onClick={onPick} className={BLOCK_TOOL_BUTTON}>
						Remplacer
					</button>
				</div>
			) : null}
			{state === 'ready' ? (
				<Button
					type="button"
					size="sm"
					variant="ghost"
					disabled={isDownloading}
					onClick={onDownload}
				>
					{isDownloading ? (
						<Loader2 className="animate-spin" />
					) : (
						<Download aria-hidden />
					)}
					Télécharger
				</Button>
			) : null}
		</div>
	)
}

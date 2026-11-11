import { Download, FileText, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
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
	return (
		<div className="flex w-full items-center gap-3 rounded-lg border bg-card px-4 py-3 text-sm">
			<FileText aria-hidden className="size-5 shrink-0 text-muted-foreground" />
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
			{state === 'ready' ? (
				<Button
					type="button"
					size="sm"
					variant="outline"
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
			{editable ? (
				<Button type="button" size="sm" variant="ghost" onClick={onPick}>
					{state === 'empty' ? 'Choisir un fichier' : 'Changer'}
				</Button>
			) : null}
		</div>
	)
}

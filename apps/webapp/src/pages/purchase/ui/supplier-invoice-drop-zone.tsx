import { AlertTriangle, CheckCircle2, Loader2, Upload, X } from 'lucide-react'
import { useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type { ImportOutcome } from '#/pages/purchase/feature/supplier-invoice-inbox-feature'

interface SupplierInvoiceDropZoneProps {
	imports: ImportOutcome[]
	isImporting: boolean
	onDismiss: (id: string) => void
	onDrop: (files: File[]) => void
}

/**
 * Drop a file, or several (#340). No dropzone library anywhere else in this
 * codebase to imitate — native drag events plus a plain multi-file
 * `<input>`, same primitives `field-photo-picker.tsx` already uses for a
 * single file.
 */
export function SupplierInvoiceDropZone({
	imports,
	isImporting,
	onDismiss,
	onDrop,
}: SupplierInvoiceDropZoneProps) {
	const [isDraggingOver, setIsDraggingOver] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)

	return (
		<div className="space-y-4 p-5">
			<button
				type="button"
				onClick={() => inputRef.current?.click()}
				onDragOver={(event) => {
					event.preventDefault()
					setIsDraggingOver(true)
				}}
				onDragLeave={() => setIsDraggingOver(false)}
				onDrop={(event) => {
					event.preventDefault()
					setIsDraggingOver(false)
					onDrop(Array.from(event.dataTransfer.files))
				}}
				className={cn(
					'flex w-full flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed p-8 text-center transition',
					isDraggingOver
						? 'border-primary bg-brand-soft'
						: 'border-muted-foreground/30 hover:border-muted-foreground/50',
				)}
			>
				<Upload className="size-6 text-muted-foreground" />
				<p className="text-sm font-medium">
					Déposez une ou plusieurs factures fournisseur ici
				</p>
				<p className="text-xs text-muted-foreground">
					Ou cliquez pour choisir des fichiers (Factur-X, PDF)
				</p>
			</button>
			<input
				ref={inputRef}
				type="file"
				multiple
				accept="application/pdf"
				className="sr-only"
				onChange={(event) => {
					const files = Array.from(event.target.files ?? [])
					if (files.length > 0) onDrop(files)
					event.target.value = ''
				}}
			/>

			{imports.length > 0 ? (
				<ul className="space-y-2">
					{imports.map((entry) => (
						<li
							key={entry.id}
							className="flex items-center gap-3 rounded-md border bg-muted/30 px-3 py-2 text-sm"
						>
							<ImportStatusIcon status={entry.status} />
							<div className="min-w-0 flex-1">
								<p className="truncate font-medium">{entry.fileName}</p>
								{entry.message ? (
									<p className="truncate text-xs text-muted-foreground">
										{entry.message}
									</p>
								) : null}
							</div>
							{entry.status !== 'pending' ? (
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									onClick={() => onDismiss(entry.id)}
								>
									<X />
									<span className="sr-only">Ignorer</span>
								</Button>
							) : null}
						</li>
					))}
				</ul>
			) : null}
			{isImporting ? null : null}
		</div>
	)
}

function ImportStatusIcon({ status }: { status: ImportOutcome['status'] }) {
	if (status === 'pending') {
		return (
			<Loader2 className="size-4 shrink-0 animate-spin text-muted-foreground" />
		)
	}
	if (status === 'created') {
		return <CheckCircle2 className="size-4 shrink-0 text-success" />
	}
	// `parse_failed` and `error` both surface the same way here: either one
	// means this file did not become an invoice, and the reason sits right
	// next to the icon either way.
	return <AlertTriangle className="size-4 shrink-0 text-destructive" />
}

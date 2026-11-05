import { Loader2, Upload } from 'lucide-react'
import { useId, useRef, useState } from 'react'
import { cn } from '#/lib/utils'

interface FileDropZoneProps {
	accept: string
	label: string
	hint: string
	isUploading: boolean
	disabled?: boolean
	compact?: boolean
	onFiles: (files: File[]) => void
}

export function FileDropZone({
	accept,
	label,
	hint,
	isUploading,
	disabled = false,
	compact = false,
	onFiles,
}: FileDropZoneProps) {
	const [isDraggingOver, setIsDraggingOver] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)
	const hintId = useId()

	return (
		<>
			<button
				type="button"
				disabled={disabled}
				aria-describedby={hintId}
				aria-busy={isUploading}
				onClick={() => inputRef.current?.click()}
				onDragOver={(event) => {
					event.preventDefault()
					setIsDraggingOver(true)
				}}
				onDragLeave={() => setIsDraggingOver(false)}
				onDrop={(event) => {
					event.preventDefault()
					setIsDraggingOver(false)
					const files = Array.from(event.dataTransfer.files)
					if (files.length > 0) onFiles(files)
				}}
				className={cn(
					'flex w-full flex-col items-center justify-center gap-1.5 rounded-lg border-2 border-dashed text-center outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50 *:pointer-events-none',
					compact ? 'px-4 py-4' : 'px-6 py-8',
					isDraggingOver
						? 'border-primary bg-primary/5'
						: 'border-muted-foreground/30 hover:border-muted-foreground/50',
				)}
			>
				{isUploading ? (
					<Loader2
						aria-hidden
						className="size-5 animate-spin text-muted-foreground"
					/>
				) : (
					<Upload aria-hidden className="size-5 text-muted-foreground" />
				)}
				{isUploading ? (
					<output className="font-medium text-sm">Envoi des fichiers…</output>
				) : (
					<span className="font-medium text-sm">{label}</span>
				)}
				<span id={hintId} className="text-muted-foreground text-xs">
					{hint}
				</span>
			</button>
			<input
				ref={inputRef}
				type="file"
				multiple
				hidden
				tabIndex={-1}
				accept={accept}
				onChange={(event) => {
					const files = Array.from(event.target.files ?? [])
					if (files.length > 0) onFiles(files)
					event.target.value = ''
				}}
			/>
		</>
	)
}

import { Loader2, Upload } from 'lucide-react'
import { useRef, useState } from 'react'
import { cn } from '#/lib/utils'

interface FileDropZoneProps {
	accept: string
	label: string
	hint: string
	isUploading: boolean
	disabled?: boolean
	onFiles: (files: File[]) => void
}

export function FileDropZone({
	accept,
	label,
	hint,
	isUploading,
	disabled = false,
	onFiles,
}: FileDropZoneProps) {
	const [isDraggingOver, setIsDraggingOver] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)

	return (
		<>
			<button
				type="button"
				disabled={disabled}
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
					'flex w-full flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed p-8 text-center transition disabled:opacity-50',
					isDraggingOver
						? 'border-primary bg-brand-soft'
						: 'border-muted-foreground/30 hover:border-muted-foreground/50',
				)}
			>
				{isUploading ? (
					<Loader2 className="size-6 animate-spin text-muted-foreground" />
				) : (
					<Upload className="size-6 text-muted-foreground" />
				)}
				<p className="text-sm font-medium">{label}</p>
				<p className="text-xs text-muted-foreground">{hint}</p>
			</button>
			<input
				ref={inputRef}
				type="file"
				multiple
				accept={accept}
				className="sr-only"
				onChange={(event) => {
					const files = Array.from(event.target.files ?? [])
					if (files.length > 0) onFiles(files)
					event.target.value = ''
				}}
			/>
		</>
	)
}

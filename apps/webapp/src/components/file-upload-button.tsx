import { Loader2, Upload } from 'lucide-react'
import { type ReactNode, useRef } from 'react'
import { Button } from '#/components/ui/button'

interface FileUploadButtonProps {
	accept: string
	multiple?: boolean
	disabled?: boolean
	isUploading?: boolean
	variant?: 'default' | 'outline' | 'ghost' | 'secondary'
	size?: 'default' | 'sm'
	children: ReactNode
	onFiles: (files: File[]) => void
}

export function FileUploadButton({
	accept,
	multiple = false,
	disabled = false,
	isUploading = false,
	variant = 'outline',
	size = 'sm',
	children,
	onFiles,
}: FileUploadButtonProps) {
	const inputRef = useRef<HTMLInputElement>(null)

	return (
		<>
			<Button
				type="button"
				variant={variant}
				size={size}
				disabled={disabled || isUploading}
				onClick={() => inputRef.current?.click()}
			>
				{isUploading ? <Loader2 className="animate-spin" /> : <Upload />}
				{children}
			</Button>
			<input
				ref={inputRef}
				type="file"
				accept={accept}
				multiple={multiple}
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

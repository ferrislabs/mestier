import { ImagePlus, LoaderCircle } from 'lucide-react'
import { useRef } from 'react'
import { Button } from '#/components/ui/button'

export function ImagePicker({
	label,
	ariaLabel,
	busy,
	accept = 'image/jpeg,image/png,image/webp',
	onPick,
}: {
	label: string
	ariaLabel?: string
	busy: boolean
	accept?: string
	onPick: (file: File) => void
}) {
	const input = useRef<HTMLInputElement>(null)
	return (
		<>
			<input
				ref={input}
				type="file"
				accept={accept}
				hidden
				tabIndex={-1}
				onChange={(event) => {
					const file = event.target.files?.[0]
					event.target.value = ''
					if (file) onPick(file)
				}}
			/>
			<Button
				type="button"
				variant="outline"
				size="sm"
				disabled={busy}
				aria-label={ariaLabel}
				onClick={() => input.current?.click()}
			>
				{busy ? (
					<LoaderCircle className="animate-spin" aria-hidden />
				) : (
					<ImagePlus aria-hidden />
				)}
				{label}
			</Button>
		</>
	)
}

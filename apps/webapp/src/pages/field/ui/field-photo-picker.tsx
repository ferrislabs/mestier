import { Camera, Loader2 } from 'lucide-react'
import type { PhotoPhase, TimeEntryPhoto } from '#/hooks/use-field'
import { PHOTO_PHASES, phaseLabel } from '../types'

interface FieldPhotoPickerProps {
	photos: TimeEntryPhoto[]
	pendingPhase: PhotoPhase | null
	onCapture: (phase: PhotoPhase, file: File) => void
}

/**
 * One capture button per phase, with a count of what is already attached.
 *
 * `capture="environment"` asks the phone for the rear camera directly, so the
 * worker gets a viewfinder rather than a file browser. Deliberately no
 * thumbnails: showing them would mean a signed url per photo, on a connection
 * that is the scarcest thing on a building site. The count is enough to know
 * the shot was taken.
 */
export function FieldPhotoPicker({
	photos,
	pendingPhase,
	onCapture,
}: FieldPhotoPickerProps) {
	return (
		<div className="grid grid-cols-3 gap-2">
			{PHOTO_PHASES.map(({ phase }) => {
				const count = photos.filter((photo) => photo.phase === phase).length
				const isPending = pendingPhase === phase

				return (
					<label
						key={phase}
						className="flex min-h-20 cursor-pointer flex-col items-center justify-center gap-1 rounded-xl border-2 border-dashed bg-card p-2 text-center transition-colors active:bg-muted"
					>
						{isPending ? (
							<Loader2 className="size-6 animate-spin text-primary" />
						) : (
							<Camera className="size-6 text-primary" />
						)}
						<span className="text-sm font-semibold">{phaseLabel(phase)}</span>
						<span className="text-xs text-muted-foreground">
							{count > 0 ? `${count} photo${count > 1 ? 's' : ''}` : 'aucune'}
						</span>
						<input
							type="file"
							accept="image/*"
							capture="environment"
							className="sr-only"
							disabled={pendingPhase !== null}
							onChange={(event) => {
								const file = event.target.files?.[0]
								if (file) onCapture(phase, file)
								event.target.value = ''
							}}
						/>
					</label>
				)
			})}
		</div>
	)
}

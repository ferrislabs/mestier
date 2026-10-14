import { ChevronLeft, ChevronRight } from 'lucide-react'
import { Button } from '#/components/ui/button'

export interface SeasonPickerProps {
	season: number
	onChange: (season: number) => void
}

export function SeasonPicker({ season, onChange }: SeasonPickerProps) {
	return (
		<div className="flex items-center gap-1">
			<Button
				type="button"
				variant="outline"
				size="icon"
				aria-label="Saison précédente"
				onClick={() => onChange(season - 1)}
			>
				<ChevronLeft />
			</Button>
			<span className="min-w-24 text-center text-sm font-semibold">
				Saison {season}
			</span>
			<Button
				type="button"
				variant="outline"
				size="icon"
				aria-label="Saison suivante"
				onClick={() => onChange(season + 1)}
			>
				<ChevronRight />
			</Button>
		</div>
	)
}

import { type ReactElement, useState } from 'react'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'

export const EMOJIS = [
	{ symbol: '👍', label: 'Pouce levé' },
	{ symbol: '✅', label: 'Validé' },
	{ symbol: '🔥', label: 'Feu' },
	{ symbol: '👀', label: 'Yeux' },
	{ symbol: '🙏', label: 'Merci' },
	{ symbol: '😄', label: 'Sourire' },
	{ symbol: '🎉', label: 'Fête' },
	{ symbol: '👌', label: 'Parfait' },
]

export function EmojiPopover({
	children,
	onSelect,
	side = 'bottom',
	align = 'center',
}: {
	children: ReactElement
	onSelect: (emoji: string) => void
	side?: 'top' | 'bottom'
	align?: 'start' | 'center' | 'end'
}) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent side={side} align={align} className="w-auto p-1.5">
				<ul aria-label="Choisir un emoji" className="grid grid-cols-4 gap-0.5">
					{EMOJIS.map((emoji) => (
						<li key={emoji.symbol}>
							<button
								type="button"
								aria-label={`Réagir avec ${emoji.symbol}`}
								title={emoji.label}
								onClick={() => {
									onSelect(emoji.symbol)
									setOpen(false)
								}}
								className="flex size-9 items-center justify-center rounded-md text-lg outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
							>
								<span aria-hidden>{emoji.symbol}</span>
							</button>
						</li>
					))}
				</ul>
			</PopoverContent>
		</Popover>
	)
}

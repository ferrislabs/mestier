import { Pin } from 'lucide-react'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { IconButton } from '#/pages/chat/ui/icon-button'

export interface PinnedMessageItem {
	id: string
	authorName: string
	whenLabel: string
	text: string
}

export function PinnedMessagesPopover({
	items,
}: {
	items: readonly PinnedMessageItem[]
}) {
	return (
		<Popover>
			<PopoverTrigger asChild>
				<IconButton label={`Messages épinglés (${items.length})`} icon={Pin} />
			</PopoverTrigger>
			<PopoverContent align="end" className="w-80 p-0">
				<h2 className="border-border border-b px-3 py-2 font-semibold text-sm">
					Messages épinglés
				</h2>
				{items.length === 0 ? (
					<p className="px-3 py-4 text-muted-foreground text-sm">
						Aucun message épinglé dans ce canal.
					</p>
				) : (
					<ul className="max-h-72 divide-y divide-border overflow-y-auto">
						{items.map((item) => (
							<li key={item.id} className="px-3 py-2.5">
								<p className="flex items-baseline gap-2 text-xs">
									<span className="font-semibold text-sm">
										{item.authorName}
									</span>
									<span className="text-muted-foreground">
										{item.whenLabel}
									</span>
								</p>
								<p className="mt-0.5 line-clamp-2 text-muted-foreground text-sm">
									{item.text}
								</p>
							</li>
						))}
					</ul>
				)}
			</PopoverContent>
		</Popover>
	)
}

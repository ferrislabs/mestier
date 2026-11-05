import { Hash, Search, Settings, Users } from 'lucide-react'
import type { ReactNode } from 'react'
import { Skeleton } from '#/components/ui/skeleton'
import { IconButton, IconToggle } from '#/pages/chat/ui/icon-button'

export interface ChatChannelHeaderUIProps {
	name?: string
	topic?: string | null
	isLoading: boolean
	searchOpen: boolean
	onSearchOpenChange: (open: boolean) => void
	membersOpen: boolean
	onMembersOpenChange: (open: boolean) => void
	pinned: ReactNode
	onOpenAdmin: () => void
}

export function ChatChannelHeaderUI({
	name,
	topic,
	isLoading,
	searchOpen,
	onSearchOpenChange,
	membersOpen,
	onMembersOpenChange,
	pinned,
	onOpenAdmin,
}: ChatChannelHeaderUIProps) {
	if (isLoading) {
		return (
			<header className="flex min-h-12 shrink-0 items-center gap-3 border-border border-b bg-background px-4 py-2">
				<Skeleton className="size-4 rounded-full" />
				<Skeleton className="h-4 w-32" />
			</header>
		)
	}

	return (
		<header className="flex min-h-12 shrink-0 items-center gap-3 border-border border-b bg-background px-4 py-2">
			<div className="flex min-w-0 flex-1 items-center gap-3">
				<h1 className="flex shrink-0 items-center gap-1 font-semibold text-base">
					<Hash aria-hidden className="size-4 text-muted-foreground" />
					{name}
				</h1>
				{topic ? (
					<p className="hidden min-w-0 truncate text-muted-foreground text-sm md:block">
						{topic}
					</p>
				) : null}
			</div>
			<div className="flex shrink-0 items-center gap-0.5">
				<IconToggle
					label="Rechercher dans le canal"
					icon={Search}
					pressed={searchOpen}
					onPressedChange={onSearchOpenChange}
				/>
				{pinned}
				<IconToggle
					label={
						membersOpen
							? 'Masquer la liste des membres'
							: 'Afficher la liste des membres'
					}
					icon={Users}
					pressed={membersOpen}
					onPressedChange={onMembersOpenChange}
					className="max-xl:hidden"
				/>
				<IconButton
					label="Paramètres du canal"
					icon={Settings}
					onClick={onOpenAdmin}
				/>
			</div>
		</header>
	)
}

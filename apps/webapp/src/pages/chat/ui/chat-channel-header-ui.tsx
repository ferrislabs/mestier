import { AlertCircle, Hash, Settings } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Skeleton } from '#/components/ui/skeleton'

export interface ChatChannelHeaderUIProps {
	name?: string
	topic?: string | null
	isLoading: boolean
	isError: boolean
	onOpenAdmin: () => void
}

export function ChatChannelHeaderUI({
	name,
	topic,
	isLoading,
	isError,
	onOpenAdmin,
}: ChatChannelHeaderUIProps) {
	if (isError) {
		return (
			<div className="flex items-center gap-2 border-b px-4 py-3 text-sm text-destructive">
				<AlertCircle className="size-4" />
				<span>Ce canal est introuvable ou n’est plus accessible.</span>
			</div>
		)
	}

	if (isLoading) {
		return (
			<div className="flex items-center gap-2 border-b px-4 py-3">
				<Skeleton className="size-4 rounded-full" />
				<Skeleton className="h-4 w-32" />
			</div>
		)
	}

	return (
		<div className="flex items-center gap-2 border-b px-4 py-3">
			<Hash className="size-4 shrink-0 text-muted-foreground" />
			<h1 className="font-semibold text-foreground">{name}</h1>
			{topic ? (
				<>
					<span className="text-muted-foreground">·</span>
					<p className="truncate text-sm text-muted-foreground">{topic}</p>
				</>
			) : null}
			<Button
				type="button"
				variant="ghost"
				size="icon"
				aria-label="Paramètres du canal"
				className="ml-auto"
				onClick={onOpenAdmin}
			>
				<Settings className="size-4" />
			</Button>
		</div>
	)
}

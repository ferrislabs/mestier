import { AlertCircle, Hash } from 'lucide-react'
import { Skeleton } from '#/components/ui/skeleton'

export interface ChatChannelHeaderUIProps {
	name?: string
	topic?: string | null
	isLoading: boolean
	isError: boolean
}

export function ChatChannelHeaderUI({
	name,
	topic,
	isLoading,
	isError,
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
		</div>
	)
}

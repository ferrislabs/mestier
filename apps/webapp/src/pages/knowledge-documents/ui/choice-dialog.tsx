import { Loader2, Search } from 'lucide-react'
import { useId } from 'react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export interface Choice {
	id: string
	label: string
	hint?: string
}

interface ChoiceDialogProps {
	open: boolean
	title: string
	description: string
	query: string | null
	items: readonly Choice[]
	isLoading: boolean
	emptyText: string
	onQueryChange: (query: string) => void
	onPick: (id: string) => void
	onOpenChange: (open: boolean) => void
}

export function ChoiceDialog({
	open,
	title,
	description,
	query,
	items,
	isLoading,
	emptyText,
	onQueryChange,
	onPick,
	onOpenChange,
}: ChoiceDialogProps) {
	const id = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				{query !== null ? (
					<div className="grid gap-1">
						<Label htmlFor={id} className="text-muted-foreground">
							Recherche
						</Label>
						<div className="relative">
							<Search
								aria-hidden
								className="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
							/>
							<Input
								id={id}
								type="search"
								value={query}
								autoComplete="off"
								className="pl-8"
								onChange={(event) => onQueryChange(event.target.value)}
							/>
						</div>
					</div>
				) : null}
				{isLoading ? (
					<div className="flex items-center gap-2 text-muted-foreground text-sm">
						<Loader2 aria-hidden className="size-4 animate-spin" />
						Chargement…
					</div>
				) : items.length === 0 ? (
					<p className="text-[13px] text-muted-foreground">{emptyText}</p>
				) : (
					<ul className="max-h-80 divide-y overflow-y-auto rounded-md border">
						{items.map((item) => (
							<li key={item.id}>
								<button
									type="button"
									onClick={() => onPick(item.id)}
									className="flex w-full items-center gap-3 px-3 py-2 text-left outline-none hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50"
								>
									<span className="min-w-0 flex-1 truncate text-sm">
										{item.label}
									</span>
									{item.hint ? (
										<span className="shrink-0 text-muted-foreground text-xs">
											{item.hint}
										</span>
									) : null}
								</button>
							</li>
						))}
					</ul>
				)}
			</DialogContent>
		</Dialog>
	)
}

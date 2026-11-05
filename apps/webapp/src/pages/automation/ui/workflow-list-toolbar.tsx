import { Search } from 'lucide-react'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	ALL_TAGS,
	type WorkflowSortKey,
} from '#/pages/automation/lib/workflow-list'

const SORT_OPTIONS: readonly { value: WorkflowSortKey; label: string }[] = [
	{ value: 'updated', label: 'Dernière modification' },
	{ value: 'name-asc', label: 'Nom (A → Z)' },
	{ value: 'name-desc', label: 'Nom (Z → A)' },
]

function isSortKey(value: string): value is WorkflowSortKey {
	return SORT_OPTIONS.some((option) => option.value === value)
}

export function WorkflowListToolbar({
	query,
	onQueryChange,
	tag,
	tags,
	onTagChange,
	sort,
	onSortChange,
}: {
	query: string
	onQueryChange: (query: string) => void
	tag: string
	tags: readonly string[]
	onTagChange: (tag: string) => void
	sort: WorkflowSortKey
	onSortChange: (sort: WorkflowSortKey) => void
}) {
	return (
		<div className="mb-4 flex flex-wrap items-center gap-2">
			<div className="relative min-w-[220px] flex-1 sm:max-w-sm">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					type="search"
					value={query}
					onChange={(event) => onQueryChange(event.target.value)}
					placeholder="Rechercher un workflow…"
					aria-label="Rechercher un workflow"
					className="pl-8"
				/>
			</div>
			<div className="ml-auto flex flex-wrap items-center gap-2">
				<Select value={tag} onValueChange={onTagChange}>
					<SelectTrigger aria-label="Filtrer par tag" className="min-w-[150px]">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL_TAGS}>Tous les tags</SelectItem>
						{tags.map((item) => (
							<SelectItem key={item} value={item}>
								{item}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<Select
					value={sort}
					onValueChange={(value) => {
						if (isSortKey(value)) onSortChange(value)
					}}
				>
					<SelectTrigger
						aria-label="Trier les workflows"
						className="min-w-[190px]"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{SORT_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		</div>
	)
}

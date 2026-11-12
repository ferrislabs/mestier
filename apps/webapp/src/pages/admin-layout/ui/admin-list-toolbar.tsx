import { Search } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'

interface AdminSearchProps {
	value: string
	placeholder: string
	onChange: (value: string) => void
}

export function AdminSearch({
	value,
	placeholder,
	onChange,
}: AdminSearchProps) {
	return (
		<div className="relative w-full max-w-sm">
			<Search
				aria-hidden
				className="-translate-y-1/2 absolute top-1/2 left-2.5 size-4 text-muted-foreground"
			/>
			<Input
				type="search"
				aria-label={placeholder}
				placeholder={placeholder}
				value={value}
				onChange={(event) => onChange(event.target.value)}
				className="pl-8"
			/>
		</div>
	)
}

interface AdminPaginationProps {
	pagination: Schemas.PaginationMetadata | null | undefined
	onPage: (page: number) => void
}

export function AdminPagination({ pagination, onPage }: AdminPaginationProps) {
	if (!pagination || pagination.is_empty) return null

	const { current_page, last_page, total, prev_page, next_page } = pagination

	return (
		<div className="flex items-center justify-between gap-3 text-muted-foreground text-sm">
			<span>
				{total != null ? `${total} résultat${total > 1 ? 's' : ''} · ` : ''}
				page {current_page}
				{last_page != null ? ` sur ${last_page}` : ''}
			</span>
			<div className="flex gap-2">
				<Button
					type="button"
					variant="outline"
					size="sm"
					disabled={prev_page == null}
					onClick={() => prev_page != null && onPage(prev_page)}
				>
					Précédente
				</Button>
				<Button
					type="button"
					variant="outline"
					size="sm"
					disabled={next_page == null}
					onClick={() => next_page != null && onPage(next_page)}
				>
					Suivante
				</Button>
			</div>
		</div>
	)
}

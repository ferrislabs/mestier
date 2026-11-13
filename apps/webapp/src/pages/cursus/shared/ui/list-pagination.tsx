import { Button } from '#/components/ui/button'

export interface ListPaginationProps {
	page: number
	pageCount: number
	onPageChange: (page: number) => void
}

export function ListPagination({
	page,
	pageCount,
	onPageChange,
}: ListPaginationProps) {
	if (pageCount <= 1) return null
	return (
		<div className="mt-3 flex items-center justify-end gap-3 text-sm">
			<Button
				type="button"
				variant="outline"
				size="sm"
				disabled={page <= 1}
				onClick={() => onPageChange(page - 1)}
			>
				Précédent
			</Button>
			<span>
				Page {page} sur {pageCount}
			</span>
			<Button
				type="button"
				variant="outline"
				size="sm"
				disabled={page >= pageCount}
				onClick={() => onPageChange(page + 1)}
			>
				Suivant
			</Button>
		</div>
	)
}

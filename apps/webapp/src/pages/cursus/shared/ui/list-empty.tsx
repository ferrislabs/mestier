import { Button } from '#/components/ui/button'
import { TableCell, TableRow } from '#/components/ui/table'

export interface ListEmptyProps {
	title: string
	hint?: string
	onReset?: () => void
}

export interface ListEmptyRowProps extends ListEmptyProps {
	colSpan: number
}

function ListEmptyContent({ title, hint, onReset }: ListEmptyProps) {
	return (
		<>
			<p className="font-medium">{title}</p>
			{hint ? (
				<p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>
			) : null}
			{onReset ? (
				<Button variant="outline" size="sm" className="mt-4" onClick={onReset}>
					Réinitialiser les filtres
				</Button>
			) : null}
		</>
	)
}

export function ListEmptyRow({ colSpan, ...content }: ListEmptyRowProps) {
	return (
		<TableRow className="hover:bg-transparent">
			<TableCell colSpan={colSpan} className="py-12 text-center">
				<ListEmptyContent {...content} />
			</TableCell>
		</TableRow>
	)
}

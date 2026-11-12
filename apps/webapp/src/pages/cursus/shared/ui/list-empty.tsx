import { Button } from '#/components/ui/button'

export interface ListEmptyProps {
	title: string
	hint?: string
	onReset?: () => void
}

export function ListEmpty({ title, hint, onReset }: ListEmptyProps) {
	return (
		<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
			<p className="font-medium">{title}</p>
			{hint ? (
				<p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>
			) : null}
			{onReset ? (
				<Button variant="outline" size="sm" className="mt-4" onClick={onReset}>
					Réinitialiser les filtres
				</Button>
			) : null}
		</div>
	)
}

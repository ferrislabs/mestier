import { cn } from '#/lib/utils'

export const DELETED_LABEL = 'Élément supprimé'

export function DeletedElement({ className }: { className?: string }) {
	return (
		<span className={cn('text-muted-foreground italic', className)}>
			{DELETED_LABEL}
		</span>
	)
}

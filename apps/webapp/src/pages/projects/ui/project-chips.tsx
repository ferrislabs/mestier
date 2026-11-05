import { Building2 } from 'lucide-react'
import { StatusBadge } from '#/pages/projects/ui/badges'

export function InternalChip() {
	return (
		<StatusBadge tone="neutral">
			<Building2 className="size-3" aria-hidden />
			Projet interne
		</StatusBadge>
	)
}

export function CustomerChip({
	customer,
}: {
	customer: { name: string } | null
}) {
	return customer ? (
		<StatusBadge tone="info">{customer.name}</StatusBadge>
	) : (
		<InternalChip />
	)
}

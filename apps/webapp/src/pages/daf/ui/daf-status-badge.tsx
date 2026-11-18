import { StatusBadge } from '#/components/ui/surface'
import {
	type DafStatus,
	statusClassName,
	statusLabel,
} from '#/pages/daf/lib/daf-status'

export function DafStatusBadge({ status }: { status: DafStatus }) {
	return (
		<StatusBadge className={statusClassName(status)}>
			{statusLabel(status)}
		</StatusBadge>
	)
}

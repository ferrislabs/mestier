import {
	AlignLeft,
	Calendar,
	CircleDot,
	Euro,
	FileText,
	Hash,
	Link2,
	ListChecks,
	type LucideIcon,
	SquareCheck,
	Type,
	User,
} from 'lucide-react'
import type { ColumnKind } from '#/pages/knowledge-tables/lib/types'

const ICONS: Record<ColumnKind, LucideIcon> = {
	text: Type,
	long_text: AlignLeft,
	number: Hash,
	currency: Euro,
	date: Calendar,
	checkbox: SquareCheck,
	single_select: CircleDot,
	multi_select: ListChecks,
	link: Link2,
	file: FileText,
	member: User,
}

export function ColumnKindIcon({
	kind,
	className,
}: {
	kind: ColumnKind
	className?: string
}) {
	const Icon = ICONS[kind]
	return <Icon aria-hidden className={className ?? 'size-3.5 shrink-0'} />
}

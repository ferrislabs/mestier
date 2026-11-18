import type { ReactNode } from 'react'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import { QUESTION_KINDS } from '#/pages/survey/lib/question-kinds'
import type { QuestionKind } from '#/pages/survey/types'

export function QuestionKindMenu({
	label,
	current,
	align = 'start',
	keepFocus = false,
	children,
	onSelect,
}: {
	label: string
	current: QuestionKind | null
	align?: 'start' | 'center' | 'end'
	keepFocus?: boolean
	children: ReactNode
	onSelect: (kind: QuestionKind) => void
}) {
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>{children}</DropdownMenuTrigger>
			<DropdownMenuContent
				align={align}
				onCloseAutoFocus={
					keepFocus ? (event) => event.preventDefault() : undefined
				}
				className="max-h-[min(70vh,32rem)] w-[min(92vw,22rem)] overflow-y-auto"
			>
				<DropdownMenuLabel>{label}</DropdownMenuLabel>
				{QUESTION_KINDS.map((entry) => (
					<DropdownMenuItem
						key={entry.kind}
						className="items-start gap-3 py-2"
						aria-current={entry.kind === current ? 'true' : undefined}
						onSelect={() => onSelect(entry.kind)}
					>
						<KindIconTile icon={entry.icon} hue={entry.hue} size="sm" />
						<span className="grid gap-0.5">
							<span className="font-medium">
								{entry.title}
								{entry.kind === current ? (
									<span className="ml-2 font-normal text-muted-foreground text-xs">
										actuel
									</span>
								) : null}
							</span>
							<span className="text-muted-foreground text-xs">
								{entry.description}
							</span>
						</span>
					</DropdownMenuItem>
				))}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

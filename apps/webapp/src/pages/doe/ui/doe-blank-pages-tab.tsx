import { Minus, Plus } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'

export const BLANK_PAGES_MIN = 1
export const BLANK_PAGES_MAX = 20

interface DoeBlankPagesTabProps {
	sectionNumber: number
	title: string
	count: number
	onCountChange: (count: number) => void
}

export function DoeBlankPagesTab({
	sectionNumber,
	title,
	count,
	onCountChange,
}: DoeBlankPagesTabProps) {
	return (
		<SectionCard>
			<SectionHeader title={`${sectionNumber}. ${title}`} />
			<div className="flex items-center gap-4 p-5">
				<span className="text-sm">Pages libres pour les notes du client</span>
				<RequirePermission permission="MANAGE_PLANNING">
					<div className="flex items-center gap-2">
						<Button
							type="button"
							variant="outline"
							size="icon-sm"
							disabled={count <= BLANK_PAGES_MIN}
							onClick={() => onCountChange(count - 1)}
						>
							<Minus />
							<span className="sr-only">Une page de moins</span>
						</Button>
						<output
							aria-label="Nombre de pages libres"
							className="w-8 text-center font-medium tabular-nums"
						>
							{count}
						</output>
						<Button
							type="button"
							variant="outline"
							size="icon-sm"
							disabled={count >= BLANK_PAGES_MAX}
							onClick={() => onCountChange(count + 1)}
						>
							<Plus />
							<span className="sr-only">Une page de plus</span>
						</Button>
					</div>
				</RequirePermission>
			</div>
		</SectionCard>
	)
}
